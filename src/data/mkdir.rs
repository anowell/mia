use super::super::CmdRunner;
use docopt::Docopt;
use algorithmia::data::HasDataPath;

static USAGE: &'static str = "
Usage:
  algo mkdir <remote>

  Create an Agorithmia data directory

  <remote>      Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: String,
}

pub struct MkDir;
impl CmdRunner for MkDir {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        Self::create_dir(&*args.arg_remote);
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