use crate::CmdRunner;
use crate::config::Profile;
use docopt::Docopt;
use algorithmia::Algorithmia;
use algorithmia::data::HasDataPath;
use std::vec::IntoIter;

static USAGE: &'static str = r##"Usage:
  algo rm <data-file>

  Removes a file from the Agorithmia Data API

  <data-file>       Specifies the Algorithmia Data URI
                    The 'data://' prefix is optional
"##;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_file: String,
}

pub struct Rm {
    client: Algorithmia,
}
impl CmdRunner for Rm {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());


        self.delete_file(&*args.arg_data_file);
    }
}

impl Rm {
    pub fn new(profile: Profile) -> Self {
        Rm {
            client: profile.client(),
        }
    }

    fn delete_file(&self, path: &str) {
        let my_file = self.client.file(path);
        match my_file.delete() {
            Ok(_) => println!("Deleted file {}", my_file.to_data_uri()),
            Err(err) => quit_err!("Error deleting file: {}", err),
        };
    }
}
