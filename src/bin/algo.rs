#![feature(slice_patterns)]
extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use algorithmia::algorithm::Algorithm;
use getopts::{Options, Matches};
use std::env;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::vec::IntoIter;
use std::iter::{Peekable, Skip};

fn print_usage(opts: &Options) -> ! {
    print!("{}", opts.usage("Usage: algo [options] USER/REPO"));
    std::process::exit(1);
}

fn die(message: &str) -> ! {
    println!("{}", message);
    std::process::exit(1);
}

fn read_file_to_string(path: &Path) -> String {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {:?}", display, why),
        Ok(file) => file,
    };

    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Err(why) => panic!("could not read {}: {:?}", display, why),
        Ok(s) => s,
    };
    data
}


fn init_service<'a>() -> Service {
    match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => Service::new(&*val),
        Err(_) => die("must set ALGORITHMIA_API_KEY"),
    }
}

// An iterator over args that supports peek and has skipped some args
type ArgIter = Peekable<Skip<IntoIter<String>>>;

fn algo_run<'a>(mut args: ArgIter, opts: Matches) {
    let algoref = match args.next() {
        Some(arg) => arg,
        None => return die("Insufficient arguments")
    };

    let algorithm = match Algorithm::from_str(&*algoref) {
        Ok(algo) => algo,
        Err(_) => return die("Invalid algorithm specification"),
    };

    // Get the --data or --file arg
    let data = match (opts.opt_str("data"), opts.opt_str("file")) {
        (Some(s), None) => s,
        (None, Some(f)) => read_file_to_string(Path::new(&*f)),
        _ => return die("must specify exactly one of -f or -d"),
    };

    // Instantiate the algorithm service
    let service = init_service();
    let algorithm_service = service.algorithm(&algorithm);

    // Execute the algorithm
    match algorithm_service.exec_raw(&*data) {
        Ok(result) => println!("{}", result),
        Err(e) => die(&*format!("HTTP ERROR: {:?}", e)),
    }
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("d", "data", "string to use as input data", "DATA");
    opts.optopt("f", "file", "file containing input data", "FILE");

    let argopts = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            return print_usage(&opts);
        }
    };

    let args_iter = argopts.free.clone().into_iter().skip(1);
    if argopts.opt_present("help") || args_iter.len() == 0 {
        return print_usage(&opts);
    }

    let mut peekable = args_iter.peekable();

    // Invoke the right module
    match peekable.peek().map(|s| &**s) {
        // Some("data") => data_main(peekable, argopts),
        Some(_) => algo_run(peekable, argopts),
        None => die("Insufficient arguments"),
    };
}