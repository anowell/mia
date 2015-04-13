use super::super::{die, init_service};
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo ls <collection>
";
/*
    TODO: add -l flag
*/

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_ls: bool,
    arg_collection: Option<String>,
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match args.arg_collection {
        Some(dir) => list_collection(&*dir),
        None => print_usage(),
    };
}

fn list_collection(path: &str) {
    let my_bucket = init_service().collection(path);
    match my_bucket.show() {
        Ok(output) => {
            println!("{}/{} - {} file(s)", output.username, output.collection_name, output.files.len());
            for f in output.files { println!("/{}", f); }
        },
        Err(why) => die(&*format!("ERROR: {:?}", why)),
    };
}