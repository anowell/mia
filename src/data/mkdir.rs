use super::super::{die, init_service};
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo mkdir <collection>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_mkdir: bool,
    arg_collection: Option<String>,
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match args.arg_collection {
        Some(dir) => create_collection(&*dir),
        None => print_usage(),
    };
}

fn create_collection(path: &str) {
    let my_bucket = init_service().collection(path);
    match my_bucket.create() {
        Ok(output) => println!("Created collection: {}/{}", output.username, output.collection_name),
        Err(why) => die(&*format!("ERROR: {:?}", why)),
    };
}