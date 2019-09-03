use {futures::prelude::*, howto::*, std::convert::TryInto, structopt::StructOpt};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Select answer in specified position
    #[structopt(short, long, default_value = "0", parse(try_from_str))]
    position: u64,
    /// Whether display only the answer link
    #[structopt(short = "l", long = "link")]
    show_link: bool,
    /// Whether display the full text of the answer
    #[structopt(short = "f", long = "full")]
    show_full: bool,
    /// Number of answers to return
    #[structopt(
        short = "n",
        long = "num-answers",
        default_value = "1",
        parse(try_from_str)
    )]
    num_answers: u64,
    query: Vec<String>,
}

async fn async_main(mut opt: Opt) {
    let query = std::mem::replace(&mut opt.query, vec![]);
    let query = query.join(" ");

    if opt.position > 0 || opt.num_answers > 1 {
        prefetch_howto(&query, (opt.position + opt.num_answers).try_into().unwrap())
            .await
            .left_stream()
    } else {
        howto(&query).await.right_stream()
    }
    .skip(opt.position)
    .take(opt.num_answers)
    .for_each(move |answer| {
        if opt.show_link {
            println!("{}", answer.link);
        } else {
            if opt.num_answers > 1 {
                println!("==== Answer from {} ====", answer.link);
            }
            if opt.show_full {
                println!("{}\n", answer.full_text);
            } else {
                println!("{}", answer.instruction);
            }
        }
        future::ready(())
    })
    .await;
}

fn main() {
    openssl_probe::init_ssl_cert_env_vars();

    let opt = Opt::from_args();

    let fut = async_main(opt);

    futures::executor::block_on(fut);
}
