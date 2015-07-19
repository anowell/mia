#![feature(scoped, semaphore)]
extern crate algorithmia;
extern crate docopt;
extern crate rustc_serialize;

use algorithmia::Algorithmia;
use std::env;

macro_rules! die {
    ($fmt:expr) => ({
        print!(concat!($fmt, "\n"));
        ::std::process::exit(1)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        print!(concat!($fmt, "\n"), $($arg)*);
        ::std::process::exit(1)
    });
}

mod data;
mod algo;

static USAGE: &'static str = "
CLI for interacting with Algorithmia

Usage:
  algo [cmd] [<args>...]
  algo [cmd] [--help]


Algorithm commands include:
  run       Runs an algorithm
  clone     Clone an algorithm
  fork      Fork an algorithm

Data commands include
  ls        List contents of a collection
  mkdir     Create a collection
  rmdir     Delete a collection
  rm        Remove a file from a collection
  upload    Upload file(s) to a collection
  download  Download file(s) from a collection
";

/* TODO: Add support for:

General commands include:
  auth    Configure authentication
Note: Add Option [--profile <profile>]

Algorithm commands include:
  view      View algorithm details (e.g. cost)
  clone     Clone an algorithm (wrapper around git clone)

Data commands include:
  download  Download file(s) from a collection
  rm        Delete file(s) in a collection
  chmod     Change permissions on a collection
*/

fn print_usage() -> ! {
    die!("{}", USAGE)
}

#[derive(RustcDecodable, Debug)]
struct MainArgs {
    arg_args: Vec<String>,
    arg_cmd: Option<String>,
    flag_h: bool,
}

fn main() {
    let mut args = env::args();
    args.next(); // drop program arg

    // Get the <cmd> arg
    let cmd = match args.next() {
        Some(c) => c,
        None => print_usage(),
    };

    // Check for cmd-specific help
    while let Some(arg) = args.next() {
        match &*arg {
            "--help" | "-h" => match &*cmd {
                "ls" => data::Ls::print_usage(),
                "mkdir" => data::MkDir::print_usage(),
                "rmdir" => data::RmDir::print_usage(),
                "upload" => data::Upload::print_usage(),
                "run" => algo::Run::print_usage(),
                _ => print_usage(),
            },
            _ => (),
        }
    };

    // Hand-off to cmd-specific   cmd_main()
    match &*cmd {
        "-h" | "--help" => print_usage(),
        "ls" => data::Ls::cmd_main(),
        "mkdir" => data::MkDir::cmd_main(),
        "rmdir" => data::RmDir::cmd_main(),
        "upload" => data::Upload::cmd_main(),
        "run" => algo::Run::cmd_main(),
        _ => algo::Run::cmd_main(),
    }
}

// TODO: figure out how to return a CmdRunner trait object to replace half the code above.
trait CmdRunner {
    fn cmd_main();
    fn get_usage() -> &'static str;

    fn print_usage() -> ! { die!("{}", Self::get_usage()) }

    fn init_client() -> Algorithmia {
        match env::var("ALGORITHMIA_API_KEY") {
            Ok(ref val) => Algorithmia::client(&**val),
            Err(_) => die!("Must set ALGORITHMIA_API_KEY"),
        }
    }

}