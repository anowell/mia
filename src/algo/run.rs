use super::super::CmdRunner;
use docopt::Docopt;

use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;
use std::vec::IntoIter;
use algorithmia::Algorithmia;
use algorithmia::algo::AlgoResponse;
use algorithmia::mime::*;
use algorithmia::error::{Error};

static USAGE: &'static str = "Usage:
  algo [run] [options] <algorithm>

  <algorithm> syntax: USERNAME/ALGONAME[/VERSION]
  Recommend specifying a version since algorithm costs can change between minor versions.

  Input Data Options:
    There are option variants for specifying the type and source of input data.
    If <file> is '-', then input data will be read from STDIN.

    Auto-Detect Data:
      -d <data>, --data <data>          If the data parses as JSON, assume JSON, else if the data
                                          is valid UTF-8, assume text, else assume binary
      -D <file>, --data-file <file>     Same as --data, but the input data is read from a file

    JSON Data:
      -j <data>, --json <data>          Algorithm input data as JSON (application/json)
      -J <file>, --json-file <file>     Same as --json, but the input data is read from a file

    Text Data:
      -t <data>, --text <data>          Algorithm input data as text (text/plain)
      -T <file>, --text-file <file>     Same as --text, but the input data is read from a file

    Binary Data:
      -b <data>, --binary <data>        Algorithm input data as binary (application/octet-stream)
      -B <data>, --binary-file <file>   Same as --data, but the input data is read from a file


  Output Options:
    By default, only the algorithm result is printed to STDOUT while additional notices may be
    printed to STDERR.

    --debug                             Print algorithm's STDOUT (author-only)
    --json-response                     Print full JSON response body instead of just the result
    -s, --silence                       Suppress printing of STDERR notices and alerts
    --time                              Print human-readable algorithm execution time

  Examples:
    algo kenny/factor/0.1.0 -t '79'                 Run algorithm with specified data input
    algo anowell/Dijkstra -J routes.json            Run algorithm with file input
    algo anowell/Dijkstra -J - < routes.json        Same as above but using STDIN
    algo opencv/SmartThumbnail -B in.jpg > out.jpg  Runs algorithm with binay data input
";

// TODO: stderr text for:
//    "Auto-detected input data as [json|text|binary]"
//    "Version not specified, using latest which may result in price changes"
//    Any alerts returned with the API metadata
// TODO: more options
//    --http-response                     Print full HTTP response including headers
//    -a --async                  Return immediately from calling the algorithm

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_run: bool,
    arg_algorithm: String,
    flag_json_response: bool,
    flag_silence: bool,
    flag_time: bool,
    flag_debug: bool,
}

pub struct Run { client: Algorithmia }
impl CmdRunner for Run {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        // We need to preprocess input args before giving other args to Docopt
        let mut input_args: Vec<InputData> = Vec::new();
        let mut other_args: Vec<String> = Vec::new();

        let mut argv_mut = argv.collect::<Vec<String>>().into_iter();
        let next_arg = |argv_iter: &mut IntoIter<String>| {
            argv_iter.next().unwrap_or_else(|| die!("Missing arg for input data option\n\n{}", USAGE))
        };
        while let Some(flag) = argv_mut.next() {
            match &*flag {
                // "-d" | "--data" => input_args.push(InputData::auto(next_arg(&mut argv_mut))),
                "-j" | "--json" => input_args.push(InputData::Json(next_arg(&mut argv_mut))),
                "-t" | "--text" => input_args.push(InputData::Text(next_arg(&mut argv_mut))),
                "-b" | "--binary" => input_args.push(InputData::Binary(next_arg(&mut argv_mut).into_bytes())),
                // -D" | "--data-file" => input_args.push(InputData::auto(read_string_src(&next_arg(&mut argv_mut)))),
                "-J" | "--json-file" => input_args.push(InputData::Json(read_string_src(&next_arg(&mut argv_mut)))),
                "-T" | "--text-file" => input_args.push(InputData::Text(read_string_src(&next_arg(&mut argv_mut)))),
                "-B" | "--binary-file" => input_args.push(InputData::Binary(read_byte_src(&next_arg(&mut argv_mut)))),
                _ => other_args.push(flag)
            };
        };

        // Finally: parse the remaining args with Docopt
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(other_args).decode())
            .unwrap_or_else(|e| e.exit());

        // Validating args and options
        if input_args.len() < 1 {
            return die!("Must specify an input data option\n\n{}", USAGE);
        } else if input_args.len() > 1 {
            return die!("Multiple input data sources is currently not supported");
        }

        // Run the algorithm
        let json_response = match self.run_algorithm(&*args.arg_algorithm, input_args.remove(0)) {
            Ok(json) => json,
            Err(err) => die!("HTTP Error: {}", err),
        };

        // Print result according to output args
        match args.flag_json_response {
            true => println!("{}", json_response),
            false => match json_response.parse::<AlgoResponse>() {
                Ok(response) => {
                    match &*response.metadata.content_type {
                        "json" => response.result_json().map(|out| {println!("{}", out);}), // TODO: call .pretty() if --pretty
                        "text" => response.result_str().map(|out| {println!("{}", out);}),
                        "binary" => response.result_bytes().map(|out| {let _ = io::stdout().write(out);}),
                        ct => return die!("Unknown result content-type: {}", ct),
                    }.unwrap_or_else(|err| return die!("Error parsing result: {}", err));
                    if args.flag_time {
                        println!("Completed in {:.1} seconds", response.metadata.duration);
                    }
                },
                Err(err) => return die!("Error parsing response: {}", err),
            }
        };
    }
}

#[derive(Debug)]
enum InputData {
    Text(String),
    Json(String),
    Binary(Vec<u8>),
}

impl Run {
    pub fn new(client: Algorithmia) -> Self { Run{ client:client } }

    fn run_algorithm(&self, algo: &str, input_data: InputData) -> Result<String, Error> {
        let algorithm = self.client.algo_from_str(algo);
        let mut response = try!(match input_data {
            InputData::Text(text) => algorithm.pipe_as(&*text, Mime(TopLevel::Text, SubLevel::Plain, vec![])),
            InputData::Json(json) => algorithm.pipe_as(&*json, Mime(TopLevel::Application, SubLevel::Json, vec![])),
            InputData::Binary(bytes) => algorithm.pipe_as(&*bytes, Mime(TopLevel::Application, SubLevel::Ext("octet-stream".into()), vec![])),
        });

        let mut json_response = String::new();
        let _ = response.read_to_string(&mut json_response);
        Ok(json_response)
    }


}

fn read_byte_src(src: &str) -> Vec<u8> {
    let mut reader = match src {
        "-" => Box::new(io::stdin()) as Box<Read>,
        s => read_file(Path::new(&s)),
    };

    let mut buf: Vec<u8> = Vec::new();
    match reader.read_to_end(&mut buf) {
        Ok(0) => die!("Error: Read 0 bytes"),
        Ok(_) => buf,
        Err(err) => die!("Error: {}", err),
    }
}

fn read_string_src(src: &str) -> String {
    let mut reader = match src {
        "-" => Box::new(io::stdin()) as Box<Read>,
        s => read_file(Path::new(&s)),
    };

    let mut buf = String::new();
    match reader.read_to_string(&mut buf) {
        Ok(0) => die!("Error: Read 0 bytes"),
        Ok(_) => {buf.pop(); buf}, // pop the EOF that turns into extra \n
        Err(err) => die!("Error: {}", err),
    }
}

fn read_file(path: &Path) -> Box<Read> {
    let display = path.display();
    let file = match File::open(&path) {
        Err(err) => die!("Error opening {}: {}", display, err),
        Ok(file) => file,
    };
    Box::new(file)
}