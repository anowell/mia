use super::super::CmdRunner;
use docopt::Docopt;
use algorithmia::Algorithmia;
use algorithmia::data::HasDataPath;
use std::vec::IntoIter;

static USAGE: &'static str = "
Usage:
  algo rm <remote>

  Removes a file from the Agorithmia Data API

  <remote>      Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: String,
}

pub struct Rm { client: Algorithmia }
impl CmdRunner for Rm {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());


        self.delete_file(&*args.arg_remote);
    }
}

impl Rm {
    pub fn new(client: Algorithmia) -> Self { Rm{ client:client } }

    fn delete_file(&self, path: &str) {
        let my_file = self.client.file(path);
        match my_file.delete() {
            Ok(_) => println!("Deleted file {}", my_file.to_data_uri()),
            Err(err) => die!("Error deleting file: {}", err),
        };
    }
}