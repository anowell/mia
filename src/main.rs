#![feature(scoped)]
#![feature(std_misc)]
extern crate algorithmia;
extern crate docopt;
extern crate rustc_serialize;

use algorithmia::Service;
use std::env;

mod data;
mod algo;

static USAGE: &'static str = "
Usage:
  algo [cmd] [<args>...]
  algo [cmd] [--help]

Commands include:
  run       Runs an algorithm
  ls        List contents of a collection
  mkdir     Create a collection
  rmdir     Delete a collection
  upload    Upload file(s) to a collection
";
/* TODO: Add support for:
  view      View algorithm details (e.g. cost)
  clone     Clone an algorithm (wrapper around git clone)
  download  Download file(s) from a collection
  rm        Delete file(s) in a collection
  chmod     Change permissions on a collection
*/

pub fn die(message: &str) -> ! {
    println!("{}", message);
    std::process::exit(1);
}

pub fn init_service() -> Service {
    match env::var("ALGORITHMIA_API_KEY") {
        Ok(ref val) => Service::new(&**val),
        Err(_) => die("Must set ALGORITHMIA_API_KEY"),
    }
}

fn print_usage() -> ! {
    die(USAGE)
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
    match args.next() {
        Some(arg) => match &*arg {
            "--help" | "-h" => match &*cmd {
                "ls" => data::ls::print_usage(),
                "mkdir" => data::mkdir::print_usage(),
                "rmdir" => data::rmdir::print_usage(),
                "upload" => data::upload::print_usage(),
                "run" => algo::run::print_usage(),
                _ => print_usage(),
            },
            _ => (),
        },
        _ => (),
    };

    // Hand-off to cmd-specific cmd_main()
    match &*cmd {
        "-h" | "--help" => print_usage(),
        "ls" => data::ls::cmd_main(),
        "rmdir" => data::rmdir::cmd_main(),
        "mkdir" => data::mkdir::cmd_main(),
        "upload" => data::upload::cmd_main(),
        "run" => algo::run::cmd_main(),
        _ => algo::run::cmd_main(),
    }

}