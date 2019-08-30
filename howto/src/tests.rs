use {
    crate::{howto, Answer},
    futures::{
        executor::{block_on, block_on_stream},
        prelude::*,
    },
    std::thread,
};

fn print_answer(answer: &Answer) {
    println!("Found answer {} from {}", answer.question_title, answer.link);
    println!("{}", answer.instruction);
}

#[test]
fn csharp_test() {
    let answers = block_on_stream(block_on(howto("file io C#")));

    let mut is_answer_exists = false;
    for answer in answers {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[test]
fn cpp_test() {
    let answers = block_on_stream(block_on(howto("file io C++")));

    let mut is_answer_exists = false;
    for answer in answers {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[test]
fn rust_test() {
    let answers = block_on_stream(block_on(howto("file io rust")));

    let mut is_answer_exists = false;
    for answer in answers {
        is_answer_exists = true;
        print_answer(&answer);
    }
    assert!(is_answer_exists);
}

#[test]
fn drop_test() {
    let mut answers = block_on(howto("file io rust"));

    let answer = block_on(answers.next()).unwrap();
    print_answer(&answer);
    drop(answers);

    println!("Waiting 10 secs...");
    thread::sleep(std::time::Duration::from_secs(10));
    println!("Success");
}
