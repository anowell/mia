use crate::CmdRunner;
use crate::config::Profile;
use docopt::Docopt;
use algorithmia::Algorithmia;
use algorithmia::data::HasDataPath;
use std::vec::IntoIter;

static USAGE: &'static str = r##"Usage:
  algo rmdir [options] <data-dir>

  Removes a directory from the Agorithmia Data API

  <data-dir>        Specifies the Algorithmia Data URI
                    The 'data://' prefix is optional

  Options:
    -f, --force     Force deletion even directory has contents

"##;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_dir: String,
    flag_force: bool,
}

pub struct RmDir {
    client: Algorithmia,
}
impl CmdRunner for RmDir {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.delete_dir(&*args.arg_data_dir, args.flag_force);
    }
}

impl RmDir {
    pub fn new(profile: Profile) -> Self {
        RmDir {
            client: profile.client(),
        }
    }

    fn delete_dir(&self, path: &str, force: bool) {
        let my_dir = self.client.dir(path);
        match my_dir.delete(force) {
            Ok(_) => println!("Deleted directory {}", my_dir.to_data_uri()),
            Err(err) => quit_err!("Error deleting directory: {}", err),
        };
    }
}
