use super::super::{die, init_service};
use docopt::Docopt;

use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

static USAGE: &'static str = "
Usage:
  algo [run] [-a] [-f <file> | -d <data>] <algorithm> [-]

  Options:
    -a --async                  Return immediately from calling the algorithm
    -f <file>, --file <file>    Use file contents as algorithm input
    -d <data>, --data <data>    Specify data to use as algorithm input
    -                           Use STDIN for input data
";

// TODO: support --async


#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_run: bool,
    arg_algorithm: Option<String>,
    flag_async: bool,
    flag_file: Option<String>,
    flag_data: Option<String>,
    cmd__: bool, // [-] stdin cmd
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let algo = match args.arg_algorithm {
        Some(algorithm) => algorithm,
        None => print_usage(),
    };

    let data = match (args.flag_data, args.flag_file, args.cmd__) {
        (Some(s), None, false) => s,
        (None, Some(f), false) => read_file_to_string(Path::new(&*f)),
        (None, None, true) => read_stdin_to_string(),
        _ => return die("must specify input data: exactly one of '-d', '-f', or '-'"),
    };

    run_algorithm(&*algo, &*data);
}

fn run_algorithm(algo: &str, input_data: &str) {
    let algorithm = match init_service().algorithm_from_str(algo) {
        Ok(a) => a,
        Err(e) => die(&*format!("Faile to parse '{}': {:?}", algo, e))
    };

    // Execute the algorithm
    match algorithm.pipe_raw(input_data, "application/json".parse().unwrap()) {
        Ok(result) => println!("{}", result),
        Err(e) => die(&*format!("HTTP ERROR: {:?}", e)),
    };
}

fn read_stdin_to_string() -> String {
    let mut buf = String::new();
    match io::stdin().read_to_string(&mut buf) {
        Ok(0) => return die("must specify an input source"),
        Ok(_) => buf,
        Err(e) => return die(&*format!("Error reading from STDIN {:?}", e)),
    }
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