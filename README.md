# howto

[![circleci](https://circleci.com/gh/pbzweihander/howto.svg?style=shield)](https://circleci.com/gh/pbzweihander/howto)
[![lib crate.io](https://img.shields.io/crates/v/howto.svg)](https://crates.io/crates/howto)
[![lib docs.rs](https://docs.rs/howto/badge.svg)](https://docs.rs/howto)
[![bin crate.io](https://img.shields.io/crates/v/howto-cli.svg)](https://crates.io/crates/howto-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Get instant coding answers via the command line. Inspired by [gleitz/howdoi](https://github.com/gleitz/howdoi)

## The Library

### Usage

```rust
let answers = howto(&query).await;
let answer = answers.next().await.unwrap();

println("{}", answer.instruction);
```

## The CLI

### Installation

#### Run with Docker

```bash
docker run --rm -it pbzweihander/howto QUERY
```

#### Install with Cargo

```bash
cargo install howto-cli
howto QUERY
```

#### Build yourself

```bash
git clone https://github.com/pbzwehiander/howto.git
cd howto/howto-cli
cargo build --release
cargo install --path .
howto QUERY
```

### Usage

```
howto-cli 0.3.0

USAGE:
    howto-cli [FLAGS] [OPTIONS] [query]...

FLAGS:
    -h, --help       Prints help information
    -f, --full       Whether display the full text of the answer
    -l, --link       Whether display only the answer link
    -V, --version    Prints version information

OPTIONS:
    -n, --num-answers <num-answers>    Number of answers to return [default: 1]
    -p, --position <position>          Select answer in specified position [default: 0]

ARGS:
    <query>...    
```
