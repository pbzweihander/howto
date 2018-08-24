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
extern crate howto;

use howto::howto;
use std::env;

fn main() {
    let query = env::args().skip(1).collect::<Vec<_>>().join(" ");

    let answers = howto(&query);

    for answer in answers.filter_map(Result::ok) {
        println("{}", answer.instruction);
    }
}
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
Usage: target/debug/howto QUERY [options]

    QUERY               the question to answer

Options:
    -h, --help          print this help message
    -p, --pos POS       select answer in specified position (default: 1)
    -a, --all           display the full text of the answer
    -l, --link          display only the answer link
    -n, --num-answers NUM_ANSWERS
                        number of answers to return (default: 1)
    -v, --version       print the current version
```
