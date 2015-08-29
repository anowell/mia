use super::super::CmdRunner;
use docopt::Docopt;
use rustc_serialize::json::Json;

use std::io::{self, Read};
use std::fs::File;
use std::path::Path;
use algorithmia::mime::*;
use algorithmia::error::Error::ApiError;

static USAGE: &'static str = "
Usage:
  algo [run] [options] <algorithm>

  <algorithm> syntax: USERNAME/ALGONAME[/VERSION]
  Recommend always specifying a version since algorithm costs can change between minor versions.
  If a version is not specified, a warning will be printed to STDERR unless '-s' has been specified

  Input Data Options:
    -d <data>, --data <data>        Algorithm input data as plain text (UTF8)
    -D <file>, --data-file <file>   Same as --data, but the input data is read from a file
                                    If <file> is '-', then input data is read from STDIN
    -j <data>, --json <data>        Algorithm input data as JSON
    -J <file>, --json-file <file>   Same as --json, but the input data is read from a file
                                    If <file> is '-', then input data is read from STDIN

  Additional Options:
    --raw                           Raw response to STDOUT (instead of only the algorithm result)

  Examples:
    algo kenny/factor/0.1.0 -d '79'                 Run algorithm with specified data input
    algo anowell/Dijkstra -J routes.json            Run algorithm with file input
    algo anowell/Dijkstra -J - < routes.json        Same as above but using STDIN
";

// TODO: support --async
//    -a --async                  Return immediately from calling the algorithm

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_run: bool,
    arg_algorithm: Option<String>,
    flag_json: Option<String>,
    flag_json_file: Option<String>,
    flag_data: Option<String>,
    flag_data_file: Option<String>,
    flag_raw: bool,
}

pub struct Run;
impl CmdRunner for Run {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        let algo = match args.arg_algorithm {
            Some(algorithm) => algorithm,
            None => Self::print_usage(),
        };

        let (content_type, data) = match (args.flag_data, args.flag_data_file, args.flag_json, args.flag_json_file) {
            (Some(s), None, None, None) => (Mime(TopLevel::Text, SubLevel::Plain, vec![]), s),
            (None, Some(s), None, None) => (Mime(TopLevel::Text, SubLevel::Plain, vec![]), Self::read_to_string(&*s)),
            (None, None, Some(s), None) => (Mime(TopLevel::Application, SubLevel::Json, vec![]), s),
            (None, None, None, Some(s)) => (Mime(TopLevel::Application, SubLevel::Json, vec![]), Self::read_to_string(&*s)),
            _ => return die!("Must specify exactly one input data option\n{}", USAGE),
        };

        let response = Self::run_algorithm(&*algo, &*data, content_type);

        match args.flag_raw {
            true => println!("{}", response),
            false => match Json::from_str(&*response) {
                Ok(json) => match json.find("result") {
                    Some(result) => match result.as_string() {
                        Some(text) => println!("{}", text),
                        None => println!("{}", result.pretty()),
                    },
                    None => die!("Error parsing response: {}", response),
                },
                Err(err) => die!("Error parsing response as JSON: {}", err),
            }
        }
    }
}

impl Run {
    fn run_algorithm(algo: &str, input_data: &str, content_type: Mime) -> String{
        let algorithm = Self::init_client().algo_from_str(algo);

        // Execute the algorithm
        match algorithm.pipe_raw(input_data, content_type) {
            Ok(result) => result,
            Err(ApiError(err)) => match err.stacktrace {
                Some(ref stacktrace) => die!("API Error: {}\n{}", err, stacktrace),
                None => die!("API Error: {}", err),
            },
            Err(err) => die!("Error calling algorithm: {}", err),
        }
    }

    fn read_to_string(source: &str) -> String {
        match source {
            "-" => Self::read_stdin_to_string(),
            s => Self::read_file_to_string(Path::new(&s[1..])),
        }
    }

    fn read_stdin_to_string() -> String {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(0) => die!("Error: reading STDIN: Read 0 bytes"),
            Ok(_) => buf,
            Err(err) => die!("Error reading STDIN: {}", err),
        }
    }

    fn read_file_to_string(path: &Path) -> String {
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(err) => die!("Error opening {}: {}", display, err),
            Ok(file) => file,
        };

        let mut data = String::new();
        match file.read_to_string(&mut data) {
            Err(err) => die!("Error reading {}: {}", display, err),
            Ok(s) => s,
        };
        data
    }
}