use super::super::CmdRunner;
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo rmdir [options] <remote>

  Removes a directory from the Agorithmia Data API

  Options:
    -f, --force                 Force deletion even directory has contents

";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: String,
    flag_force: bool,
}

pub struct RmDir;
impl CmdRunner for RmDir {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        Self::delete_dir(&*args.arg_remote, args.flag_force);
    }
}

impl RmDir {
    fn delete_dir(path: &str, force: bool) {
        let my_dir = Self::init_client().dir(path);
        match my_dir.delete(force) {
            Ok(_) => println!("Deleted directory {}", my_dir.to_data_uri()),
            // TODO: Improve error message when delete failed for lack of --force
            Err(why) => die!("ERROR: {:?}", why),
        };
    }
}