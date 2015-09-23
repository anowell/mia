use super::super::CmdRunner;
use docopt::Docopt;

use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;
use std::vec::IntoIter;
use rustc_serialize::json::Json;
use algorithmia::Algorithmia;
use algorithmia::algo::AlgoResponse;
use algorithmia::mime::*;
use algorithmia::client::Response;
use algorithmia::error::{Error};

static USAGE: &'static str = "Usage:
  algo [run] [options] <algorithm>

  <algorithm> syntax: USERNAME/ALGONAME[/VERSION]
  Recommend specifying a version since algorithm costs can change between minor versions.

  Input Data Options:
    There are option variants for specifying the type and source of input data.
    If <file> is '-', then input data will be read from STDIN.

    Auto-Detect Data:
      -d, --data <data>             If the data parses as JSON, assume JSON, else if the data
                                      is valid UTF-8, assume text, else assume binary
      -D, --data-file <file>        Same as --data, but the input data is read from a file

    JSON Data:
      -j, --json <data>             Algorithm input data as JSON (application/json)
      -J, --json-file <file>        Same as --json, but the input data is read from a file

    Text Data:
      -t, --text <data>             Algorithm input data as text (text/plain)
      -T, --text-file <file>        Same as --text, but the input data is read from a file

    Binary Data:
      -b, --binary <data>           Algorithm input data as binary (application/octet-stream)
      -B, --binary-file <file>      Same as --data, but the input data is read from a file


  Output Options:
    By default, only the algorithm result is printed to STDOUT while additional notices may be
    printed to STDERR.

    --debug                         Print algorithm's STDOUT (author-only)
    --response-body                 Print HTTP response body (replaces result)
    --response                      Print full HTTP response including headers (replaces result)
    -s, --silence                   Suppress any output not explicitly requested (except result)
    -m, --meta                      Print human-readable selection of metadata (e.g. duration)
    -o, --output <file>             Print result to a file, implies --meta

  Examples:
    algo kenny/factor/0.1.0 -t '79'                   Run algorithm with specified data input
    algo anowell/Dijkstra -J routes.json              Run algorithm with file input
    algo anowell/Dijkstra -J - < routes.json          Same as above but using STDIN
    algo opencv/SmartThumbnail -B in.png -o out.png   Runs algorithm with binay data input
";

// TODO: stderr text for:
//    "Auto-detected input data as [json|text|binary]"
//    "Version not specified, using latest which may result in price changes"
// TODO: more options
//    --result-json               Force result to be printed as JSON (base64 for binary)
//    -a --async                  Return immediately from calling the algorithm

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_run: bool,
    arg_algorithm: String,
    flag_response_body: bool,
    flag_response: bool,
    flag_silence: bool,
    flag_meta: bool,
    flag_debug: bool,
    flag_output: Option<String>,
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
                "-d" | "--data" => input_args.push(InputData::auto(&mut next_arg(&mut argv_mut).as_bytes())),
                "-j" | "--json" => input_args.push(InputData::Json(next_arg(&mut argv_mut))),
                "-t" | "--text" => input_args.push(InputData::Text(next_arg(&mut argv_mut))),
                "-b" | "--binary" => input_args.push(InputData::Binary(next_arg(&mut argv_mut).into_bytes())),
                "-D" | "--data-file" => input_args.push(InputData::auto(&mut get_src(&next_arg(&mut argv_mut)))),
                "-J" | "--json-file" => input_args.push(InputData::json(&mut get_src(&next_arg(&mut argv_mut)))),
                "-T" | "--text-file" => input_args.push(InputData::text(&mut get_src(&next_arg(&mut argv_mut)))),
                "-B" | "--binary-file" => input_args.push(InputData::binary(&mut get_src(&next_arg(&mut argv_mut)))),
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

        // Open up an output device for the result/response
        let mut output = OutputDevice::new(&args.flag_output);

        // Run the algorithm
        let mut response = self.run_algorithm(&*args.arg_algorithm, input_args.remove(0));

        // Read JSON response - scoped so that we can re-borrow response
        let mut json_response = String::new();
        {
            match response.read_to_string(&mut json_response) {
                Ok(json) => json,
                Err(err) => die!("Read error: {}", err)
            };
        }

        // Handle --response and --response-body (ignoring other flags)
        if args.flag_response || args.flag_response_body {
            if args.flag_response {
                let preamble = format!("{} {}\n{}", response.version, response.status, response.headers);
                output.writeln(preamble.as_bytes());
            };
            output.writeln(json_response.as_bytes());
        } else {
            match json_response.parse::<AlgoResponse>() {
                Ok(response) => {
                    // Printing any API alerts
                    if let Some(ref alerts) = response.metadata.alerts {
                        if !args.flag_silence {
                            for alert in alerts {
                                let _ = io::stderr().write(alert.as_bytes());
                                let _ = io::stderr().write(b"\n");
                            }
                        }
                    }

                    // Printing algorithm stdout
                    if let Some(ref stdout) = response.metadata.stdout {
                        if args.flag_debug {
                            println!("{}", stdout);
                        }
                    }

                    // Printing metadata
                    if args.flag_meta || (args.flag_output.is_some() && !args.flag_silence) {
                        println!("Completed in {:.1} seconds", response.metadata.duration);
                    }

                    // Smart output of result
                    match &*response.metadata.content_type {
                        "json" => response.result_json().map(|out| output.writeln(out.as_bytes())),
                        "text" => response.result_str().map(|out| output.writeln(out.as_bytes())),
                        "binary" => response.result_bytes().map(|out| output.write(out)),
                        ct => return die!("Unknown result content-type: {}", ct),
                    }.unwrap_or_else(|err| return die!("Error parsing result: {}", err));
                },
                Err(err) => return die!("Error parsing response: {}", err),
            };
        }

    }
}

#[derive(Debug)]
enum InputData {
    Text(String),
    Json(String),
    Binary(Vec<u8>),
}

impl InputData {

    // Auto-detect the InputData type
    // 1. Json if it parses as JSON
    // 2. Text if it parses as UTF-8
    // 3. Fallback to binary
    fn auto(reader: &mut Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => match Json::from_str(&data) {
                Ok(_) => InputData::Json(data),
                Err(_) => InputData::Text(data),
            },
            Err(_) => {
                let mut bytes: Vec<u8> = Vec::new();
                match reader.read_to_end(&mut bytes) {
                    Ok(_) => InputData::Binary(bytes),
                    Err(err) => die!("Read error: {}", err),
                }
            }
        }
    }

    fn text(reader: &mut Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Text(data),
            Err(err) => die!("Read error: {}", err),
        }
    }

    fn json(reader: &mut Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Json(data),
            Err(err) => die!("Read error: {}", err),
        }
    }

    fn binary(reader: &mut Read) -> InputData {
        let mut bytes: Vec<u8> = Vec::new();
        match reader.read_to_end(&mut bytes) {
            Ok(_) => InputData::Binary(bytes),
            Err(err) => die!("Read error: {}", err),
        }
    }
}


// The device specified by --output flag
// Only the result or response is written to this device
struct OutputDevice {
    writer: Box<Write>
}

impl OutputDevice {
    fn new(output_dest: &Option<String>) -> OutputDevice {
        match output_dest {
            &Some(ref file_path) => match File::create(file_path) {
                Ok(buf) => OutputDevice{ writer: Box::new(buf) },
                Err(err) => die!("Unable to create file: {}", err),
            },
            &None => OutputDevice{ writer: Box::new(io::stdout()) },
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match self.writer.write(bytes) {
            Ok(_) => (),
            Err(err) => die!("Error writing output: {}", err),
        }
    }

    fn writeln(&mut self, bytes: &[u8]) {
        self.write(bytes);
        self.write(b"\n");
    }
}

impl Run {
    pub fn new(client: Algorithmia) -> Self { Run{ client:client } }

    fn run_algorithm(&self, algo: &str, input_data: InputData) -> Response { // Result<String, Error> {
        let algorithm = self.client.algo(algo);
        let result = match input_data {
            InputData::Text(text) => algorithm.pipe_as(&*text, Mime(TopLevel::Text, SubLevel::Plain, vec![])),
            InputData::Json(json) => algorithm.pipe_as(&*json, Mime(TopLevel::Application, SubLevel::Json, vec![])),
            InputData::Binary(bytes) => algorithm.pipe_as(&*bytes, Mime(TopLevel::Application, SubLevel::Ext("octet-stream".into()), vec![])),
        };

        match result {
            Ok(response) => response,
            Err(err) => die!("HTTP Error: {}", err),
        }
    }
}


fn get_src(src: &str) -> Box<Read> {
    match src {
        "-" => Box::new(io::stdin()) as Box<Read>,
        s => open_file(Path::new(&s)),
    }
}

fn open_file(path: &Path) -> Box<Read> {
    let display = path.display();
    let file = match File::open(&path) {
        Err(err) => die!("Error opening {}: {}", display, err),
        Ok(file) => file,
    };
    Box::new(file)
}