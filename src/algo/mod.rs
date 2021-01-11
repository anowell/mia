pub use self::clone::GitClone;
pub use self::run::Run;

mod clone;
mod run;

use crate::{color_choice, BRIGHT_RED, GRAY};
use algorithmia::algo::{AlgoResponse, Response};
use rustc_serialize::json::Json;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::vec::IntoIter;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

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
    fn auto(reader: &mut dyn Read) -> InputData {
        let mut bytes: Vec<u8> = Vec::new();
        if let Err(err) = reader.read_to_end(&mut bytes) {
            quit_err!("Read error: {}", err);
        }

        match String::from_utf8(bytes) {
            Ok(data) => match Json::from_str(&data) {
                Ok(_) => InputData::Json(data),
                Err(_) => InputData::Text(data),
            },
            Err(not_utf8) => InputData::Binary(not_utf8.into_bytes()),
        }
    }

    fn text(reader: &mut dyn Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Text(data),
            Err(err) => quit_err!("Read error: {}", err),
        }
    }

    fn json(reader: &mut dyn Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Json(data),
            Err(err) => quit_err!("Read error: {}", err),
        }
    }

    fn binary(reader: &mut dyn Read) -> InputData {
        let mut bytes: Vec<u8> = Vec::new();
        match reader.read_to_end(&mut bytes) {
            Ok(_) => InputData::Binary(bytes),
            Err(err) => quit_err!("Read error: {}", err),
        }
    }
}

// The device specified by --output flag
// Only the result or response is written to this device
struct OutputDevice {
    writer: Box<dyn Write>,
}

impl OutputDevice {
    fn new(output_dest: &Option<String>) -> OutputDevice {
        match *output_dest {
            Some(ref file_path) => match File::create(file_path) {
                Ok(buf) => OutputDevice {
                    writer: Box::new(buf),
                },
                Err(err) => quit_err!("Unable to create file: {}", err),
            },
            None => OutputDevice {
                writer: Box::new(io::stdout()),
            },
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match self.writer.write(bytes) {
            Ok(_) => (),
            Err(err) => quit_err!("Error writing output: {}", err),
        }
    }

    fn writeln(&mut self, bytes: &[u8]) {
        self.write(bytes);
        self.write(b"\n");
    }
}

fn get_src(src: &str) -> Box<dyn Read> {
    match src {
        "-" => Box::new(io::stdin()) as Box<dyn Read>,
        s => open_file(Path::new(&s)),
    }
}

fn open_file(path: &Path) -> Box<dyn Read> {
    let display = path.display();
    let file = match File::open(&path) {
        Err(err) => quit_err!("Error opening {}: {}", display, err),
        Ok(file) => file,
    };
    Box::new(file)
}

struct ResponseConfig {
    flag_response_body: bool,
    flag_response: bool,
    flag_silence: bool,
    flag_debug: bool,
    flag_output: Option<String>,
}

fn display_response(mut response: Response, config: ResponseConfig) {
    // Open up an output device for the result/response
    let mut output = OutputDevice::new(&config.flag_output);
    let mut t_err = StandardStream::stderr(color_choice());

    // Read JSON response - scoped so that we can re-borrow response
    let mut json_response = String::new();
    {
        if let Err(err) = response.read_to_string(&mut json_response) {
            quit_err!("Error reading response: {}", err)
        };
    }

    // Handle --response and --response-body (ignoring other flags)
    if config.flag_response || config.flag_response_body {
        if config.flag_response {
            let preamble = format!(
                "{:?} {}\n{:?}",
                response.version(),
                response.status(),
                response.headers()
            );
            output.writeln(preamble.as_bytes());
        };
        output.writeln(json_response.as_bytes());
    } else {
        match json_response.parse::<AlgoResponse>() {
            Ok(response) => {
                // Printing any API alerts
                if let Some(ref alerts) = response.metadata.alerts {
                    if !config.flag_silence {
                        let _ = t_err.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
                        for alert in alerts {
                            let _ = writeln!(t_err, "{}", alert);
                        }
                        let _ = t_err.reset();
                    }
                }

                // Printing algorithm stdout
                if let Some(ref stdout) = response.metadata.stdout {
                    if config.flag_debug {
                        let _ = t_err.set_color(ColorSpec::new().set_fg(Some(GRAY)));
                        let _ = writeln!(t_err, "{}", stdout);
                        let _ = t_err.reset();
                    }
                }

                // Printing metadata
                if !config.flag_silence {
                    let _ = t_err.set_color(ColorSpec::new().set_fg(Some(GRAY)));
                    let _ = writeln!(
                        t_err,
                        "Completed in {:.1} seconds",
                        response.metadata.duration
                    );
                    let _ = t_err.reset();
                }

                // Smart output of result
                match response.result.as_string() {
                    Some(s) => output.writeln(s.as_bytes()),
                    None => output.write(response.result.as_bytes().unwrap()),
                };
            }
            Err(ref error) if error.api_error().is_some() => {
                let err = error.api_error().unwrap();
                let mut t_err = StandardStream::stderr(color_choice());
                let _ = t_err.set_color(ColorSpec::new().set_fg(Some(BRIGHT_RED)));
                let _ = writeln!(t_err, "API error: {}", err.message);
                let _ = t_err.reset();

                if let Some(ref trace) = err.stacktrace {
                    eprintln!("{}", trace)
                }
                ::std::process::exit(1);
            }
            Err(err) => {
                quit_err!(
                    "Failed to parse algorithm response (debug with --response-body)\n{}",
                    err
                )
            }
        };
    }
}

// separates input-defining args from other args
fn split_args(argv: IntoIter<String>, usage: &'static str) -> (Vec<InputData>, Vec<String>) {
    let mut input_args: Vec<InputData> = Vec::new();
    let mut other_args: Vec<String> = Vec::new();

    let mut argv_mut = argv.collect::<Vec<String>>().into_iter();
    let next_arg = |argv_iter: &mut IntoIter<String>| {
        argv_iter
            .next()
            .unwrap_or_else(|| quit_msg!("Missing arg for input data option\n\n{}", usage))
    };
    while let Some(flag) = argv_mut.next() {
        match &*flag {
            "-d" | "--data" => {
                input_args.push(InputData::auto(&mut next_arg(&mut argv_mut).as_bytes()))
            }
            "-j" | "--json" => input_args.push(InputData::Json(next_arg(&mut argv_mut))),
            "-t" | "--text" => input_args.push(InputData::Text(next_arg(&mut argv_mut))),
            "-b" | "--binary" => {
                input_args.push(InputData::Binary(next_arg(&mut argv_mut).into_bytes()))
            }
            "-D" | "--data-file" => {
                input_args.push(InputData::auto(&mut get_src(&next_arg(&mut argv_mut))))
            }
            "-J" | "--json-file" => {
                input_args.push(InputData::json(&mut get_src(&next_arg(&mut argv_mut))))
            }
            "-T" | "--text-file" => {
                input_args.push(InputData::text(&mut get_src(&next_arg(&mut argv_mut))))
            }
            "-B" | "--binary-file" => {
                input_args.push(InputData::binary(&mut get_src(&next_arg(&mut argv_mut))))
            }
            _ => other_args.push(flag),
        };
    }

    // Validating args and options
    if input_args.len() < 1 {
        quit_msg!("Must specify an input data option\n\n{}", usage);
    } else if input_args.len() > 1 {
        quit_msg!("Multiple input data sources is currently not supported");
    }

    (input_args, other_args)
}
