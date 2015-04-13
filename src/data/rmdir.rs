use super::super::{die, init_service};
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo rmdir <collection>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_rmdir: bool,
    arg_collection: Option<String>,
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match args.arg_collection {
        Some(dir) => delete_collection(&*dir),
        None => print_usage(),
    };
}

fn delete_collection(path: &str) {
    let my_bucket = init_service().collection(path);
    match my_bucket.delete() {
        Ok(_) => println!("Deleted collection"),
        Err(why) => die(&*format!("ERROR: {:?}", why)),
    };
}