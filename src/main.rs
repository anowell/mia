#![feature(slice_patterns)]
#![feature(std_misc)]
extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use getopts::{Options, Matches};
use std::env;
use std::vec::IntoIter;
use std::iter::{Peekable, Skip};

mod data;
mod algo;

// An iterator over args that supports peek and has skipped some args
pub type ArgIter = Peekable<Skip<IntoIter<String>>>;

pub struct ArgParser {
    options: Options,
    matches: Matches,
    arg_iter: ArgIter,
}

fn print_usage(opts: &Options) -> ! {
    let brief = r#"
Usage: algo [options] USER/REPO
       algo data CMD [CMD_ARGS...]

The first form runs an algorithm.
The latter form interacts with the data API: see `algo data --help` for more info
"#;

    println!("{}", opts.usage(brief));
    std::process::exit(1);
}

pub fn die(message: &str) -> ! {
    println!("{}", message);
    std::process::exit(1);
}

pub fn init_service<'a>() -> Service {
    match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => Service::new(&*val),
        Err(_) => die("Must set ALGORITHMIA_API_KEY"),
    }
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("d", "data", "string to use as input data", "DATA");
    opts.optopt("f", "file", "file containing input data", "FILE");
    opts.optopt("c", "concurrency", "concurrent threads to use for uploading files", "COUNT");

    let matches = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            return print_usage(&opts);
        }
    };

    let arg_iter = matches.free.clone().into_iter().skip(1);
    if matches.opt_present("help") || arg_iter.len() == 0 {
        return print_usage(&opts);
    }

    let mut parsed_args = ArgParser {
        options: opts,
        matches: matches,
        arg_iter: arg_iter.peekable(),
    };

    // Invoke the right module
    match parsed_args.arg_iter.peek().map(|s| &**s) {
        Some("data") => { parsed_args.arg_iter.next(); data::exec_main(parsed_args) },
        Some(_) => algo::exec_main(parsed_args),
        None => die("Insufficient arguments"),
    };

}