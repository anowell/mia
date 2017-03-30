use super::super::CmdRunner;
use docopt::Docopt;

use algorithmia::{Algorithmia, ApiAuth};
use algorithmia::algo::Response;
use std::process::{Command, Stdio};
use std::{thread, time};
use std::io::{self, Write, BufRead, BufReader};
use std::vec::IntoIter;
use hyper::client::Client;
use term::{self, color};
use isatty::stderr_isatty;
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

    --debug                         Print algorithm's STDOUT (default for 'algo runlocal')
    --no-debug                      Don't print algorithm's STDOUT (default for 'algo run')
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
    flag_no_debug: bool,
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

        if args.flag_debug && args.flag_no_debug {
            quit_msg!("Cannot specify both --debug and --no-debug");
        }

        // --debug can override --silence, but the lack of --debug respects --silence
        let debug = args.flag_debug || !(args.flag_no_debug || args.flag_silence);

        // Start `algo serve` if not already running
        let client = Client::new();
        let started_server = match client.get("http://0.0.0.0:9999").send() {
            Ok(_) => {
                println!("Using previously running instance of `algo serve`");
                false
            }
            Err(_) => {
                self.serve_algorithm(debug, args.flag_silence);
                true
            }
        };


        // Run the algorithm
        let response = self.call_algorithm(input_args.remove(0));

        // Only stop `algo serve` if we started it
        if started_server {
            self.terminate_algorithm();
        }

        let config = ResponseConfig {
            flag_response_body: args.flag_response_body,
            flag_response: args.flag_response,
            flag_silence: args.flag_silence,
            flag_debug: false, // use real-time debug instead of response debug
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
            client: Algorithmia::client_with_url("http://localhost:9999", ApiAuth::None),
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
            Err(err) => quit_err!("HTTP Error: {}", err),
        }
    }

    fn serve_algorithm(&self, debug: bool, silence: bool) {
        let mut child = Command::new("algo")
            .arg("serve")
            .arg("--profile")
            .arg(&self.serve_profile)
            .env_remove("RUST_LOG")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| quit_msg!("Failed to run `algo serve`"));


        // Block until langserver is alive (most of this time is spent in `bin/build`)
        let client = Client::new();
        let mut t_err = term::stderr().unwrap();
        let mut i = 0;
        while let Err(_) = client.get("http://0.0.0.0:9999").send() {
            if !silence {
                let _ = t_err.carriage_return();
                let imod = i % 10;
                let _ = write!(t_err, "[{0:1$}*{0:2$}] Building... ", "", imod, 9 - imod);
                let _ = io::stdout().flush();
            }
            thread::sleep(time::Duration::from_millis(100));
            i += 1;
            if i > 10 * 60 * 2 {
                quit_msg!("Failed to wait for algorithm. Try running `algo serve` manually.")
            }
        }
        if !silence {
            let _ = t_err.carriage_return();
            let _ = write!(t_err, "{:24}", "");
            let _ = t_err.carriage_return();
            let _ = io::stdout().flush();
        }

        let stderr = child.stderr.take()
            .unwrap_or_else(|| quit_msg!("Failed to open algorithm's STDOUT"));
        let _ = thread::spawn(move || {
            if debug {
                let stderr_reader = BufReader::new(stderr);
                let line_iter = stderr_reader.lines()
                    .filter_map(Result::ok)
                    .filter_map(|line| {
                        let mut parts = line.splitn(4, ' ');
                        let color = match parts.nth(1) {
                            Some("ALGOOUT") => color::BRIGHT_BLACK,
                            Some("ALGOERR") => color::BRIGHT_RED,
                            _ => return None,
                        };
                        parts.nth(1).map(|msg| (color, msg.to_owned()))
                    });
                for (color, line) in line_iter {
                    // TODO color based on line.1
                    if stderr_isatty() { let _ = t_err.fg(color); }
                    let _ = writeln!(t_err, "{}", line);
                    if stderr_isatty() { let _ = t_err.reset(); }
                }
            }
            let _ = child.wait();
        });
    }

    fn terminate_algorithm(&self) {
        let client = Client::new();
        let _ = client.delete("http://0.0.0.0:9999").send();
        let mut t_err = term::stderr().unwrap();
        let _ = t_err.carriage_return();
        let _ = t_err.delete_line();
    }
}

