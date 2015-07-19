use super::super::CmdRunner;
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo mkdir <collection>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_collection: Option<String>,
}

pub struct MkDir;
impl CmdRunner for MkDir {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        match args.arg_collection {
            Some(dir) => Self::create_dir(&*dir),
            None => Self::print_usage(),
        };
    }

}

impl MkDir {
    fn create_dir(path: &str) {
        let my_dir = Self::init_client().dir(path);
        match my_dir.create() {
            Ok(_) => println!("Created directory: {}", my_dir.to_data_uri()),
            Err(why) => die!("ERROR: {:?}", why),
        };
    }
}