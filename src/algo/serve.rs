use super::super::CmdRunner;
use docopt::Docopt;

use std::process::Command;
use std::env;
use std::vec::IntoIter;
use langserver::{LangServer, LangServerMode};
use hyper::server::Server;

static USAGE: &'static str = "Usage:
  algo serve [options]

  This will start a minimal server to serve the algorithm in the current directory..

  Note: This does not currently work for Java or Scala algorithms.

  Options:
    -c <runtime_image>              Containerize server using specific docker image
    -z <zipfile>                    Zipfile to serve
";


#[derive(RustcDecodable, Debug)]
struct Args {
    // arg_c: String,
    // arg_z: String,
}

pub struct Serve;
impl CmdRunner for Serve {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());


        // Serve the algorithm
        self.serve_algorithm();
    }
}

impl Serve {
    pub fn new() -> Self {
        Serve
    }

    fn serve_algorithm(&self) {
        let mut path = env::current_dir().expect("Failed to get current dir");
        path.push("bin/build");

        let mut child = Command::new("bin/build")
                            .spawn()
                            .unwrap_or_else(|_| { die!("Failed to run `bin/build`")});
        let _ = child.wait();

        let langserver = LangServer::start(LangServerMode::Sync, None)
            .unwrap_or_else(|err| { die!("Failed to start LangServer: {}", err)});

        let _ = Server::http("0.0.0.0:9999")
            .and_then(|s| s.handle(langserver))
            .map(|_listener| {
                println!("Listening on port 9999.");
                // TODO: tear down listener cleanly when algorithm completes
            });
    }


}

