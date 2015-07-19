use super::super::CmdRunner;
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo rmdir <collection>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_collection: Option<String>,
}

pub struct RmDir;
impl CmdRunner for RmDir {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        match args.arg_collection {
            Some(dir) => Self::delete_dir(&*dir),
            None => Self::print_usage(),
        };
    }
}

impl RmDir {
    fn delete_dir(path: &str) {
        let my_dir = Self::init_client().dir(path);
        match my_dir.delete() {
            Ok(_) => println!("Deleted directory {}", my_dir.to_data_uri()),
            Err(why) => die!("ERROR: {:?}", why),
        };
    }
}