//! # howto
//!
//! Instant coding answers with Google and StackOverflow.
//! Inspired by [gleitz/howdoi](https://github.com/gleitz/howdoi).
//!
//! ## Usage
//!
//! ```
//! # use futures::prelude::*;
//! let answers = howto::howto("file io rust").wait();
//!
//! for answer in answers.filter_map(Result::ok) {
//!     println!("Answer from {}\n{}", answer.link, answer.instruction);
//! }
//! ```

use {
    failure::Error,
    futures::prelude::*,
    lazy_static::lazy_static,
    reqwest::r#async::Client,
    scraper::{Html, Selector},
    std::thread,
    tokio,
};

/// Struct containing the answer of given query.
#[derive(Debug, Clone)]
pub struct Answer {
    pub question_title: String,
    pub link: String,
    pub full_text: String,
    pub instruction: String,
}

fn get(url: &str) -> impl Future<Item = String, Error = Error> {
    let client = Client::new();
    let resp_future = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0",
        )
        .send();

    resp_future.map_err(Into::into).and_then(|resp| {
        resp.into_body().concat2().map_err(Into::into).map(|chunk| {
            let v = chunk.to_vec();
            String::from_utf8_lossy(&v).to_string()
        })
    })
}

fn get_stackoverflow_links(query: &str) -> impl Future<Item = Vec<String>, Error = Error> {
    lazy_static! {
        static ref LINK_SELECTOR: Selector = Selector::parse(".r>a").unwrap();
    }

    let url = format!(
        "https://www.google.com/search?q=site:stackoverflow.com {}",
        query,
    );
    let query = query.to_string();

    get(&url)
        .map(|content| {
            let html = Html::parse_document(&content);

            let links: Vec<_> = html
                .select(&LINK_SELECTOR)
                .filter_map(|e| e.value().attr("href"))
                .map(ToString::to_string)
                .collect();

            links
        })
        .map_err(move |e| e.context(format!("error in query {}", query)).into())
}

fn get_answer(link: &str) -> impl Future<Item = Option<Answer>, Error = Error> {
    lazy_static! {
        static ref TITLE_SELECTOR: Selector = Selector::parse("#question-header>h1").unwrap();
        static ref ANSWER_SELECTOR: Selector = Selector::parse(".answer").unwrap();
        static ref TEXT_SELECTOR: Selector = Selector::parse(".post-text>*").unwrap();
        static ref PRE_INSTRUCTION_SELECTOR: Selector = Selector::parse("pre").unwrap();
        static ref CODE_INSTRUCTION_SELECTOR: Selector = Selector::parse("code").unwrap();
    }

    let url = format!("{}?answerstab=votes", link);
    let link = link.to_string();
    let link1 = link.clone();

    get(&url)
        .map(|content| Html::parse_document(&content))
        .map(|html| {
            let title = html
                .select(&TITLE_SELECTOR)
                .next()
                .map(|title| title.text().collect::<Vec<_>>().join(""));
            let rest = html.select(&ANSWER_SELECTOR).next().and_then(|answer| {
                answer
                    .select(&PRE_INSTRUCTION_SELECTOR)
                    .next()
                    .or_else(|| answer.select(&CODE_INSTRUCTION_SELECTOR).next())
                    .map(|e| e.text().collect::<Vec<_>>().join(""))
                    .map(|instruction| {
                        let full_text = answer
                            .select(&TEXT_SELECTOR)
                            .flat_map(|e| e.text())
                            .collect::<Vec<_>>()
                            .join("");

                        (link, instruction, full_text)
                    })
            });

            title.and_then(|question_title| {
                rest.map(|(link, instruction, full_text)| Answer {
                    question_title,
                    link,
                    instruction,
                    full_text,
                })
            })
        })
        .map_err(move |e| e.context(format!("error in link {}", link1)).into())
}

/// Query function. Give query to this function and thats it! Google and StackOverflow will do the rest.
pub fn howto(query: &str) -> impl Stream<Item = Answer, Error = Error> {
    use futures::{future::ok, stream::futures_ordered, sync::mpsc::channel};

    let (sender, receiver) = channel::<Result<Answer, Error>>(8);

    let links_future = get_stackoverflow_links(query);

    let answers_stream = links_future
        .map(|v| futures_ordered(v.into_iter().map(|link| get_answer(&link))))
        .flatten_stream()
        .filter_map(|o| o);

    let answers_future = answers_stream
        .then(ok::<_, Error>)
        .forward(sender)
        .then(|_| ok(()));

    thread::spawn(move || {
        tokio::run(answers_future);
    });

    receiver.map_err(|_| unreachable!()).and_then(|r| r)
}

#[test]
fn csharp_test() {
    let answers = howto("file io C#").wait();

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn cpp_test() {
    let answers = howto("file io C++").wait();

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn rust_test() {
    let answers = howto("file io rust").wait();

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn drop_test() {
    let mut answers = howto("file io rust").wait();

    let answer = answers.next().unwrap().unwrap();
    println!("Answer from: {}\n{}", answer.link, answer.instruction);
    drop(answers);

    thread::sleep(std::time::Duration::from_secs(5));
}
