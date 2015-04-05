#![feature(exit_status)]
#![feature(slice_patterns)]
extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use algorithmia::algorithm::Algorithm;
use getopts::Options;
use std::env;
use std::io::Read;
use std::fs::File;
use std::path::Path;

fn print_usage(opts: &Options) {
    print!("{}", opts.usage("Usage: algo [options] USER/REPO"));
    env::set_exit_status(1);
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

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("d", "data", "string to use as input data", "DATA");
    opts.optopt("f", "file", "file containing input data", "FILE");

    let argopts = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            print_usage(&opts);
            return;
        }
    };

    let api_key = match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            println!("Must set ALGORITHMIA_API_KEY");
            print_usage(&opts);
            return;
        }
    };


    let mut args_iter = argopts.free.clone().into_iter().skip(1);
    if argopts.opt_present("help") || args_iter.len() == 0 {
        print_usage(&opts);
        return;
    }

    // Get the USERNAME/ALGORITHM[/VERSION] arg
    let algo_arg = args_iter.next();
    let algorithm = match algo_arg.as_ref().and_then(|algo| {
        Algorithm::from_str(&*algo).ok()
    }){
        Some(algo) => algo,
        None => {
            println!("Did not correctly specify USERNAME/ALGORITHM[/VERSION]");
            print_usage(&opts);
            return;
        }
    };

    // Get the --data or --file arg
    let data = match (argopts.opt_str("data"), argopts.opt_str("file")) {
        (Some(s), None) => s,
        (None, Some(f)) => read_file_to_string(Path::new(&*f)),
        _ => {
            println!("Must specify exactly one of -f or -d");
            print_usage(&opts);
            return;
        }
    };

    // Instantiate the algorithm service
    let service = Service::new(&*api_key);
    let algorithm_service = service.algorithm(&algorithm);

    // Execute the algorithm
    let output = match algorithm_service.exec_raw(&*data) {
        Ok(result) => result,
        Err(e) => format!("HTTP ERROR: {:?}", e),
    };

    println!("{}", output);
}