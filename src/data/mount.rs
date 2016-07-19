extern crate algorithmia_fuse;

use super::super::CmdRunner;
use docopt::Docopt;
use algorithmia::Algorithmia;
use self::algorithmia_fuse::{AlgoFs, MountOptions};
use std::vec::IntoIter;


static USAGE: &'static str = "Usage:
  algo mount [options] <path>

  Mount Algorithmia data to a local filesystem

  <path>            Specifies the local path where Algorithmia data will be mounted
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_path: String,
    flag_max_inodes: Option<u32>,
    flag_max_cache: Option<u32>,
}

pub struct Mount { client: Algorithmia }

impl CmdRunner for Mount {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.mount(&args.arg_path);
    }
}

impl Mount {
    pub fn new(client: Algorithmia) -> Self { Mount{ client:client } }

    fn mount(&self, path: &str) {

        let options = MountOptions::new(&path);;
        AlgoFs::mount(options, self.client.clone());

    }
}
