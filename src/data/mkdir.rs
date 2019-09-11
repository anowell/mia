use crate::CmdRunner;
use crate::config::Profile;
use docopt::Docopt;
use algorithmia::Algorithmia;
use algorithmia::data::{HasDataPath, DataAcl};
use std::vec::IntoIter;

static USAGE: &'static str = r##"Usage:
  algo mkdir <data-dir>

  Create an Agorithmia data directory

  <data-dir>    Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
"##;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_dir: String,
}

pub struct MkDir {
    client: Algorithmia,
}
impl CmdRunner for MkDir {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.create_dir(&*args.arg_data_dir);
    }
}

impl MkDir {
    pub fn new(profile: Profile) -> Self {
        MkDir {
            client: profile.client(),
        }
    }

    fn create_dir(&self, path: &str) {
        let my_dir = self.client.dir(path);
        match my_dir.create(DataAcl::default()) {
            Ok(_) => println!("Created directory: {}", my_dir.to_data_uri()),
            Err(err) => quit_err!("Error creating directory: {}", err),
        };
    }
}
