use super::super::{die, init_service};
use docopt::Docopt;

use std::io::Read;
use std::fs::File;
use std::path::Path;

static USAGE: &'static str = "
Usage:
  algo run [-a] [-f <file> | -d <data>] <algorithm> [-]

  Options:
    -a --async                  Return immediately from calling the algorithm
    -f <file>, --file <file>    Use file contents as algorithm input
    -d <data>, --data <data>    Specify data to use as algorithm input
    -                           Use stdin as the algorithm input
";

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_run: bool,
    arg_algorithm: Option<String>,
    flag_async: bool,
    flag_file: Option<String>,
    flag_data: Option<String>,
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());


    let data = match (args.flag_data, args.flag_file) {
        (Some(s), None) => s,
        (None, Some(f)) => read_file_to_string(Path::new(&*f)),
        _ => return die("must specify exactly one of -f or -d"),
    };

    match args.arg_algorithm {
        Some(algo) => run_algorithm(&*algo, &*data),
        None => print_usage(),
    }
}

fn run_algorithm(algo: &str, input_data: &str) {
    let algorithm = match init_service().algorithm_from_str(algo) {
        Ok(a) => a,
        Err(e) => die(&*format!("PARSE ERROR: {:?}", e))
    };

    // Execute the algorithm
    match algorithm.exec_raw(input_data) {
        Ok(result) => println!("{}", result),
        Err(e) => die(&*format!("HTTP ERROR: {:?}", e)),
    };
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