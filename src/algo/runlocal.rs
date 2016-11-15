use super::super::CmdRunner;
use docopt::Docopt;

use algorithmia::{Algorithmia, ApiAuth};
use algorithmia::algo::Response;
use std::process::{Command, Stdio};
use std::{thread, time};
use std::io::{self, Write};
use std::vec::IntoIter;
use hyper::client::Client;
use term;
use super::{InputData, ResponseConfig, split_args, display_response};

static USAGE: &'static str = r##"Usage:
  algo runlocal [options]

  This will test your algorithm locally, similar to how `algo run` works but using
  an algorithm from a local directory.

  Note: This does not currently work for Java or Scala algorithms.

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
    -o, --output <file>             Print result to a file

  Examples:
    algo runlocal -d 'foo'          Tests the algorithm in the current directory with 'foo' as input
"##;


#[derive(RustcDecodable, Debug)]
struct Args {
    flag_response_body: bool,
    flag_response: bool,
    flag_silence: bool,
    flag_debug: bool,
    flag_output: Option<String>,
}

pub struct RunLocal {
    client: Algorithmia,
    serve_profile: String,
}
impl CmdRunner for RunLocal {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        // We need to preprocess input args before giving other args to Docopt
        let (mut input_args, other_args) = split_args(argv, USAGE);

        // Parse the remaining args with Docopt
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(other_args).decode())
            .unwrap_or_else(|e| e.exit());

        // Run the algorithm
        self.serve_algorithm();
        let response = self.call_algorithm(input_args.remove(0));
        self.terminate_algorithm();

        let config = ResponseConfig {
            flag_response_body: args.flag_response_body,
            flag_response: args.flag_response,
            flag_silence: args.flag_silence,
            flag_debug: args.flag_debug,
            flag_output: args.flag_output,
        };

        display_response(response, config);
    }
}

impl RunLocal {
    pub fn new(profile: &str) -> Self {
        RunLocal {
            // Will serve with profile, and point client to `algo serve`
            serve_profile: profile.to_owned(),
            client: Algorithmia::alt_client("http://localhost:9999", ApiAuth::None),
        }
    }

    fn call_algorithm(&self, input_data: InputData) -> Response {
        let algorithm = self.client.algo("local/local");

        let result = match input_data {
            InputData::Text(text) => algorithm.pipe_as(&*text, mime!(Text / Plain)),
            InputData::Json(json) => algorithm.pipe_as(&*json, mime!(Application / Json)),
            InputData::Binary(bytes) => {
                algorithm.pipe_as(&*bytes, mime!(Application / OctetStream))
            }
        };

        match result {
            Ok(response) => response,
            Err(err) => die!("HTTP Error: {}", err),
        }
    }

    fn serve_algorithm(&self) {
        let mut child = Command::new("algo")
            .arg("serve")
            .arg("--profile")
            .arg(&self.serve_profile)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|_| die!("Failed to run `algo serve`"));

        let _ = thread::spawn(move || {
            let _ = child.wait();
        });

        // Block until langserver is alive (most of this time is spent in `bin/build`)
        let client = Client::new();
        let mut t_err = term::stderr().unwrap();
        let mut i = 0;
        while let Err(_) = client.get("http://0.0.0.0:9999").send() {
            let _ = t_err.carriage_return();
            let imod = i % 10;
            let _ = write!(t_err, "[{0:1$}*{0:2$}] Building... ", "", imod, 9 - imod);
            let _ = io::stdout().flush();
            thread::sleep(time::Duration::from_millis(100));
            i += 1;
            if i > 10 * 60 * 2 {
                die!("Failed to wait for algorithm. Try running `algo serve` manually.")
            }
        }
    }

    fn terminate_algorithm(&self) {
        let client = Client::new();
        let _ = client.delete("http://0.0.0.0:9999").send();
        let mut t_err = term::stderr().unwrap();
        let _ = t_err.carriage_return();
        let _ = t_err.delete_line();
    }
}

