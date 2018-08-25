//! # howto
//!
//! Instant coding answers with Google and StackOverflow.
//! Inspired by [gleitz/howdoi](https://github.com/gleitz/howdoi).
//!
//! ## Usage
//!
//! ```
//! let answers = howto::howto("file io rust");
//!
//! for answer in answers.filter_map(Result::ok) {
//!     println!("Answer from {}\n{}", answer.link, answer.instruction);
//! }
//! ```

extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate scraper;
extern crate tokio;
extern crate url;

pub use failure::Error;

use futures::future::ok;
use futures::stream::futures_ordered;
use futures::{Future, Stream};
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use scraper::{Html, Selector};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use url::form_urlencoded::byte_serialize;

/// Struct containing the answer of given query.
#[derive(Debug, Clone)]
pub struct Answer {
    pub link: String,
    pub full_text: String,
    pub instruction: String,
}

/// Blocking iterator that gets answers from Stream.
pub struct Answers {
    inner: Receiver<Result<Answer, Error>>,
}

impl Iterator for Answers {
    type Item = Result<Answer, Error>;

    fn next(&mut self) -> Option<Result<Answer, Error>> {
        self.inner.recv().ok()
    }
}

fn get(url: &str) -> impl Future<Item = String, Error = Error> {
    let req = Request::get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0",
        ).body(Body::empty())
        .expect("request construction failed");

    let connector = HttpsConnector::new(4).expect("TLS initialization failed");

    let client = Client::builder().build(connector);

    let resp_future = client.request(req);

    resp_future
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
        "https://www.google.com/search?q=site:stackoverflow.com%20{}",
        query
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
        }).map_err(move |e| e.context(format!("error in query {}", query)).into())
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
    let link1 = link.clone();

    get(&url)
        .map(|content| Html::parse_document(&content))
        .map(|html| {
            html.select(&ANSWER_SELECTOR).next().and_then(|answer| {
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

                        Answer {
                            link,
                            instruction,
                            full_text,
                        }
                    })
            })
        }).map_err(move |e| e.context(format!("error in link {}", link1)).into())
}

/// Query function. Give query to this fuction ans thats it. Google and StackOverflow do the rest.
pub fn howto(query: &str) -> Answers {
    let query: String = byte_serialize(query.as_bytes()).collect();
    let (sender, receiver) = channel::<Result<Answer, Error>>();

    let links_future = get_stackoverflow_links(&query);

    let answers_stream = links_future
        .map(|v| futures_ordered(v.into_iter().map(|link| get_answer(&link))))
        .flatten_stream()
        .filter_map(|o| o);

    let answers_future = answers_stream
        .map_err({
            let sender = sender.clone();
            move |e| {
                let _ = sender.send(Err(e));
            }
        }).for_each(move |a| {
            let _ = sender.send(Ok(a));
            ok(())
        });

    thread::spawn(move || {
        tokio::run(answers_future);
    });

    Answers { inner: receiver }
}

#[test]
fn csharp_test() {
    let answers = howto("file io C#");

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn cpp_test() {
    let answers = howto("file io C++");

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn rust_test() {
    let answers = howto("file io rust");

    for answer in answers {
        let answer = answer.unwrap();
        println!("Answer from: {}\n{}", answer.link, answer.instruction);
    }
}

#[test]
fn drop_test() {
    let mut answers = howto("file io rust");

    let answer = answers.next().unwrap().unwrap();
    println!("Answer from: {}\n{}", answer.link, answer.instruction);
    drop(answers);

    thread::sleep(std::time::Duration::from_secs(5));
}
