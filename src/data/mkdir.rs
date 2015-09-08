use super::super::CmdRunner;
use docopt::Docopt;
use algorithmia::Algorithmia;
use algorithmia::data::HasDataPath;
use std::vec::IntoIter;

static USAGE: &'static str = "Usage:
  algo mkdir <data-dir>

  Create an Agorithmia data directory

  <data-dir>    Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_dir: String,
}

pub struct MkDir { client: Algorithmia }
impl CmdRunner for MkDir {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.create_dir(&*args.arg_data_dir);
    }

}

impl MkDir {
    pub fn new(client: Algorithmia) -> Self { MkDir{ client:client } }

    fn create_dir(&self, path: &str) {
        let my_dir = self.client.dir(path);
        match my_dir.create() {
            Ok(_) => println!("Created directory: {}", my_dir.to_data_uri()),
            Err(err) => die!("Error creating directory: {}", err),
        };
    }
}