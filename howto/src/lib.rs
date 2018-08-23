extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;
extern crate scraper;
extern crate slugify;

pub use failure::Error;

use futures::stream::futures_ordered;
use futures::{Future, Stream};
use hyper::{Body, Client, Request};
use scraper::{Html, Selector};
use slugify::slugify;

#[derive(Debug, Clone)]
pub struct Answer {
    pub link: String,
    pub full_text: String,
    pub instruction: Option<String>,
}

pub struct Answers {
    inner_stream: Box<Stream<Item = Answer, Error = Error>>,
}

impl Answers {
    pub fn into_stream(self) -> impl Stream<Item = Answer, Error = Error> {
        self.inner_stream
    }
}

impl IntoIterator for Answers {
    type Item = Result<Answer, Error>;
    type IntoIter = futures::stream::Wait<Box<Stream<Item = Answer, Error = Error>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner_stream.wait()
    }
}

fn get(url: &str) -> impl Future<Item = String, Error = Error> {
    let req = Request::get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0",
        ).body(Body::empty())
        .unwrap();

    let client = Client::new();

    client
        .request(req)
        .map(|resp| resp.into_body())
        .and_then(|body| {
            body.fold(vec![], |mut acc, chunk| -> Result<_, hyper::Error> {
                acc.extend_from_slice(&chunk);
                Ok(acc)
            })
        }).map_err(Into::<Error>::into)
        .and_then(|v| String::from_utf8(v).map_err(Into::<Error>::into))
}

fn get_stackoverflow_links(query: &str) -> impl Future<Item = Vec<String>, Error = Error> {
    lazy_static! {
        static ref LINK_SELECTOR: Selector = Selector::parse(".r>a").unwrap();
    }

    let url = format!(
        "http://www.google.com/search?q=site:stackoverflow.com%20{}",
        query
    );

    get(&url).map(|content| {
        let html = Html::parse_document(&content);

        let links: Vec<_> = html
            .select(&LINK_SELECTOR)
            .filter_map(|e| e.value().attr("href"))
            .map(ToString::to_string)
            .collect();

        links
    })
}

fn get_answer(link: &str) -> impl Future<Item = Option<Answer>, Error = Error> {
    lazy_static! {
        static ref ANSWER_SELECTOR: Selector = Selector::parse(".answer").unwrap();
        static ref TEXT_SELECTOR: Selector = Selector::parse(".post-text>*").unwrap();
        static ref PRE_INSTRUCTION_SELECTOR: Selector = Selector::parse("pre").unwrap();
        static ref CODE_INSTRUCTION_SELECTOR: Selector = Selector::parse("code").unwrap();
    }

    let url = format!("{}?answerstab=votes", link);
    let link = link.to_string();

    get(&url)
        .map(|content| Html::parse_document(&content))
        .map(|html| {
            html.select(&ANSWER_SELECTOR).next().map(|answer| {
                let instruction = answer
                    .select(&PRE_INSTRUCTION_SELECTOR)
                    .next()
                    .or_else(|| answer.select(&CODE_INSTRUCTION_SELECTOR).next())
                    .map(|e| e.text().collect::<Vec<_>>().join(""));
                let full_text = answer
                    .select(&TEXT_SELECTOR)
                    .flat_map(|e| e.text())
                    .collect::<Vec<_>>()
                    .join("");

                Answer {
                    link,
                    instruction,
                    full_text,
                }
            })
        })
}

pub fn howto(query: &str) -> Answers {
    let query = slugify!(query, separator = "+");

    let links_future = get_stackoverflow_links(&query);

    let answers_stream = links_future
        .map(|v| futures_ordered(v.into_iter().map(|link| get_answer(&link))))
        .flatten_stream()
        .filter_map(|o| o);

    Answers {
        inner_stream: Box::new(answers_stream),
    }
}
