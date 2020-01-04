use crate::{howto, Answer};
use tokio::stream::StreamExt;

fn print_answer(answer: &Answer) {
    println!(
        "Found answer {} from {}",
        answer.question_title, answer.link,
    );
    println!("{}", answer.instruction);
}

#[tokio::test]
async fn csharp_test() {
    let mut answers = howto("file io C#").await;

    let mut is_answer_exists = false;
    while let Some(answer) = answers.next().await {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[tokio::test]
async fn cpp_test() {
    let mut answers = howto("file io C++").await;

    let mut is_answer_exists = false;
    while let Some(answer) = answers.next().await {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[tokio::test]
async fn rust_test() {
    let mut answers = howto("file io rust").await;

    let mut is_answer_exists = false;
    while let Some(answer) = answers.next().await {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[tokio::test]
async fn drop_test() {
    let mut answers = howto("file io rust").await;

    let answer = answers.next().await.unwrap();
    print_answer(&answer);
    drop(answers);

    println!("Waiting 10 secs...");
    tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
    println!("Success");
}
