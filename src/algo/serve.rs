use super::super::CmdRunner;
use docopt::Docopt;
use std::vec::IntoIter;

static USAGE: &'static str = "Usage:
  algo serve [options] [<path>]

  This will start a minimal server to serve the algorithm in the current directory..

  Note: This does not currently work for Java or Scala algorithms.

  Options:
    -c, --container <runtime_image>              Containerize server using specific docker image
    -p, --port <port>                            Port to listen on [default: 9999]
";


#[derive(RustcDecodable, Debug)]
struct Args {
    // TODO: support using algorithm.zip path
    arg_path: Option<String>,
    flag_port: u32, // arg_container: Option<String>, // TODO:
}

pub struct Serve;
impl CmdRunner for Serve {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        // Serve the algorithm
        native::serve_algorithm(args.flag_port as u16, args.arg_path.as_ref());
    }
}

impl Serve {
    pub fn new() -> Self {
        Serve
    }
}

#[cfg(not(target_os = "windows"))]
mod native {
    use std::env;
    use std::process::Command;
    use hyper::server::Server;
    use langserver::{LangServer, LangServerMode};

    pub fn serve_algorithm(port: u16, path: Option<&String>) {
        if let Some(p) = path {
            env::set_current_dir(p)
                .unwrap_or_else(|err| die!("Failed to set working directory: {}", err));
        }

        let mut child = Command::new("bin/build")
            .spawn()
            .unwrap_or_else(|_| die!("Failed to run `bin/build`"));
        let _ = child.wait();

        let langserver = LangServer::start(LangServerMode::Sync, None)
            .unwrap_or_else(|err| die!("Failed to start LangServer: {}", err));

        let _ = Server::http(("0.0.0.0", port))
            .and_then(|s| s.handle(langserver))
            .map(|_listener| {
                println!("Listening on port {}.", port);
                // TODO: tear down listener cleanly when algorithm completes
            });
    }
}

#[cfg(target_os = "windows")]
mod native {
    pub fn serve_algorithm(port: u16, path: Option<&String>) {
        unimplemented!()
    }
}
