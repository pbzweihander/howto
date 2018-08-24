extern crate getopts;
extern crate howto;

use getopts::Options;
use howto::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum ShowOption {
    CodeOnly,
    LinkOnly,
    FullText,
}

#[derive(Debug)]
struct Config {
    query: String,
    position: usize,
    show_option: ShowOption,
    num_answers: usize,
}

fn help_message(program: &str, opts: &Options) -> String {
    let brief = format!(
        "Usage: {} QUERY [options]\n\n    QUERY               the question to answer",
        program
    );
    opts.usage(&brief)
}

fn get_config_from_args(args: Vec<String>) -> Result<Config, String> {
    let mut opts = Options::new();
    let program = args[0].clone();

    opts.optflag("h", "help", "print this help message")
        .optopt(
            "p",
            "pos",
            "select answer in specified position (default: 1)",
            "POS",
        ).optflag("a", "all", "display the full text of the answer")
        .optflag("l", "link", "display only the answer link")
        .optopt(
            "n",
            "num-answers",
            "number of answers to return (default: 1)",
            "NUM_ANSWERS",
        ).optflag("v", "version", "print the current version");

    macro_rules! bail {
        () => {
            return Err(help_message(&program, &opts));
        };
    }
    macro_rules! ensure {
        ($b:expr) => {
            if !$b {
                bail!();
            }
        };
    }

    let matches = opts.parse(&args[1..]);

    ensure!(matches.is_ok());
    let matches = matches.unwrap();
    ensure!(!matches.free.is_empty());
    ensure!(!matches.opt_present("help"));
    if matches.opt_present("version") {
        return Err(VERSION.to_string());
    }

    macro_rules! get_opt_or_default {
        ($n:expr, $d:expr) => {
            match matches.opt_get($n) {
                Ok(o) => o.unwrap_or($d),
                Err(_) => bail!(),
            };
        };
    }

    let query = matches.free.join(" ");
    ensure!(!query.is_empty());

    let position = get_opt_or_default!("pos", 1);
    ensure!(position >= 1);

    let num_answers = get_opt_or_default!("num-answers", 1);
    ensure!(num_answers >= 1);

    let show_option = match (matches.opt_present("link"), matches.opt_present("all")) {
        (false, false) => ShowOption::CodeOnly,
        (true, false) => ShowOption::LinkOnly,
        (false, true) => ShowOption::FullText,
        (true, true) => bail!(),
    };

    Ok(Config {
        query,
        position,
        show_option,
        num_answers,
    })
}

fn main() {
    let config = get_config_from_args(std::env::args().collect::<Vec<_>>());
    if let Err(e) = config {
        eprintln!("{}", e);
        return;
    }
    let config = config.unwrap();

    let answers = howto(&config.query);

    answers
        .skip(config.position - 1)
        .take(config.num_answers)
        .for_each(|answer| match answer {
            Err(e) => eprintln!("{}", e),
            Ok(answer) => match config.show_option {
                ShowOption::CodeOnly => {
                    if config.num_answers > 1 {
                        println!("=== Answers from {} ===", answer.link);
                    }
                    println!("{}\n", answer.instruction);
                }
                ShowOption::LinkOnly => println!("{}", answer.link),
                ShowOption::FullText => {
                    if config.num_answers > 1 {
                        println!("=== Answers from {} ===", answer.link);
                    }
                    println!("{}\n", answer.full_text);
                }
            },
        });
}
