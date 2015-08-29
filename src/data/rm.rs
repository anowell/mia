use super::super::CmdRunner;
use docopt::Docopt;
use algorithmia::data::HasDataPath;

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

pub struct Rm;
impl CmdRunner for Rm {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());


        Self::delete_file(&*args.arg_remote);
    }
}

impl Rm {
    fn delete_file(path: &str) {
        let my_file = Self::init_client().file(path);
        match my_file.delete() {
            Ok(_) => println!("Deleted file {}", my_file.to_data_uri()),
            Err(err) => die!("Error deleting file: {}", err),
        };
    }
}