//! # howto
//!
//! Instant coding answers with Google and StackOverflow.
//! Inspired by [gleitz/howdoi](https://github.com/gleitz/howdoi).
//!
//! ## Usage
//!
//! ```
//! # use futures::prelude::*;
//! # async move {
//! let answers = howto::howto("file io rust").await;
//!
//! answers.for_each(|answer| {
//!     println!("Answer from {}\n{}", answer.link, answer.instruction);
//!     future::ready(())
//! }).await;
//! # };
//! ```

#![feature(async_closure, try_blocks)]

#[cfg(test)]
mod tests;

use {
    failure::{ensure, format_err, Fallible},
    futures::prelude::*,
    lazy_static::lazy_static,
    scraper::{Html, Selector},
};

/// Struct containing the answer of given query.
#[derive(Debug, Clone)]
pub struct Answer {
    pub question_title: String,
    pub link: String,
    pub full_text: String,
    pub instruction: String,
}

async fn get(url: &str) -> Fallible<String> {
    let mut resp = surf::get(url)
        .set_header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0",
        )
        .await
        .map_err(|e| failure::Error::from_boxed_compat(e))?;

    ensure!(
        resp.status().is_success(),
        format_err!("Request error: {}", resp.status())
    );

    Ok(resp
        .body_string()
        .await
        .map_err(|e| failure::Error::from_boxed_compat(e))?)
}

async fn get_stackoverflow_links(query: &str) -> Fallible<Vec<String>> {
    lazy_static! {
        static ref LINK_SELECTOR: Selector = Selector::parse(".r>a").unwrap();
    }

    let url = format!(
        "https://www.google.com/search?q=site:stackoverflow.com {}",
        query,
    );

    let content = get(&url).await?;
    let html = Html::parse_document(&content);

    let links: Vec<_> = html
        .select(&LINK_SELECTOR)
        .filter_map(|e| e.value().attr("href"))
        .map(ToString::to_string)
        .filter(|link| link.starts_with("https://stackoverflow.com/"))
        .collect();

    Ok(links)
}

async fn get_answer(link: &str) -> Fallible<Answer> {
    lazy_static! {
        static ref TITLE_SELECTOR: Selector = Selector::parse("#question-header>h1").unwrap();
        static ref ANSWER_SELECTOR: Selector = Selector::parse(".answer").unwrap();
        static ref TEXT_SELECTOR: Selector = Selector::parse(".post-text>*").unwrap();
        static ref PRE_INSTRUCTION_SELECTOR: Selector = Selector::parse("pre").unwrap();
        static ref CODE_INSTRUCTION_SELECTOR: Selector = Selector::parse("code").unwrap();
    }
    macro_rules! unwrap_or_bail {
        ($o:expr) => {
            $o.ok_or_else(|| format_err!("Cannot parse StackOverflow"))?
        };
    };

    let url = format!("{}?answerstab=votes", link);
    let link = link.to_string();

    let content = get(&url).await?;
    let html = Html::parse_document(&content);

    let title_html = unwrap_or_bail!(html.select(&TITLE_SELECTOR).next());
    let question_title = title_html.text().collect::<Vec<_>>().join("");

    let answer = unwrap_or_bail!(html.select(&ANSWER_SELECTOR).next());

    let instruction_html = unwrap_or_bail!(answer
        .select(&PRE_INSTRUCTION_SELECTOR)
        .next()
        .or_else(|| answer.select(&CODE_INSTRUCTION_SELECTOR).next()));
    let instruction = instruction_html.text().collect::<Vec<_>>().join("");
    let full_text = answer
        .select(&TEXT_SELECTOR)
        .flat_map(|e| e.text())
        .collect::<Vec<_>>()
        .join("");

    Ok(Answer {
        question_title,
        link,
        instruction,
        full_text,
    })
}

/// Query function. Give query to this function and thats it! Google and StackOverflow will do the rest.
pub async fn howto(query: &str) -> stream::BoxStream<'_, Answer> {
    let links = get_stackoverflow_links(query).await.unwrap_or_default();

    stream::iter(links)
        .filter_map(async move |link| get_answer(&link).await.ok())
        .boxed()
}

/// Prefetch n queries with `FuturesOrdered`, and then others.
pub async fn prefetch_howto(query: &str, n: usize) -> stream::BoxStream<'_, Answer> {
    let mut links = get_stackoverflow_links(query).await.unwrap_or_default();

    let others = if links.len() < n {
        vec![]
    } else {
        links.split_off(n)
    };

    let prefetch_stream = links
        .into_iter()
        .map(async move |link| get_answer(&link).await.ok())
        .collect::<stream::FuturesOrdered<_>>()
        .filter_map(future::ready);
    let others_stream =
        stream::iter(others).filter_map(async move |link| get_answer(&link).await.ok());

    prefetch_stream.chain(others_stream).boxed()
}
