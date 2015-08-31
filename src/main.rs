extern crate algorithmia;
extern crate chan;
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
  algo [cmd] [options] [<args>...]
  algo [cmd] [--help]


Algorithm commands include:
  run       Runs an algorithm

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
  fork      Fork an algorithm

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
    let mut args = env::args().peekable();
    args.next(); // drop program arg

    let cmd = args.peek().unwrap_or_else(|| { print_usage() }).clone();

    // Search for --help flag
    while let Some(arg) = args.next() {
        if &*arg == "--help" {
            print_cmd_usage(&*cmd)
        }
    };

    run_cmd(&*cmd)
}

fn run_cmd(cmd: &str) {
    let client = match env::var("ALGORITHMIA_API_KEY") {
        Ok(ref val) => Algorithmia::client(&**val),
        Err(_) => die!("Must set ALGORITHMIA_API_KEY"),
    };

    match cmd {
        "ls" => run(data::Ls::new(client)),
        "mkdir" => run(data::MkDir::new(client)),
        "rmdir" => run(data::RmDir::new(client)),
        "rm" => run(data::Rm::new(client)),
        "upload" => run(data::Upload::new(client)),
        "download" => run(data::Download::new(client)),
        "run" => run(algo::Run::new(client)),
        _ => run(algo::Run::new(client)),
    };
}

fn run<T: CmdRunner>(runner: T) {
  runner.cmd_main();
}

fn print_cmd_usage(cmd: &str) -> ! {
    match cmd {
        "ls" => data::Ls::print_usage(),
        "mkdir" => data::MkDir::print_usage(),
        "rmdir" => data::RmDir::print_usage(),
        "rm" => data::Rm::print_usage(),
        "upload" => data::Upload::print_usage(),
        "download" => data::Download::print_usage(),
        "run" => algo::Run::print_usage(),
        _ => print_usage(),
    };
}


trait CmdRunner {
    fn cmd_main(&self);
    fn get_usage() -> &'static str;

    fn print_usage() -> ! { die!("{}", Self::get_usage()) }
}