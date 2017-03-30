use super::super::CmdRunner;
use docopt::Docopt;
use std::vec::IntoIter;
use std::env;
use std::net::TcpListener;
use ::Profile;
use env_logger::{LogBuilder};

static USAGE: &'static str = "Usage:
  algo serve [options] [<path>]

  This will start a minimal server to serve the algorithm in the current directory..

  Note: This does not currently work for Java or Scala algorithms.

  Options:
    --no-build                           Skip building the project (reuse previous build artifact)
    -p, --port <port>                    Port to listen on [default: 9999]
";


#[derive(RustcDecodable, Debug)]
struct Args {
    // TODO: support using algorithm.zip path
    arg_path: Option<String>,
    flag_port: u32,
    flag_no_build: bool,
}

pub struct Serve;
impl CmdRunner for Serve {
    fn get_usage() -> &'static str {
        USAGE
    }

    #[cfg(target_os = "windows")]
    fn cmd_main(&self, argv: IntoIter<String>) {
        quit_msg!("algo serve is not currently supported on Windows")
    }

    #[cfg(not(target_os = "windows"))]
    fn cmd_main(&self, argv: IntoIter<String>) {
        // Setup logging for langserver
        let mut builder = LogBuilder::new();
        builder.format(|record| format!("{} {}", record.level(), record.args()));
        if let Ok(log_var) = env::var("RUST_LOG") {
            builder.parse(&log_var);
        } else {
            builder.parse("error,langserver=info/ALGO(OUT|ERR)");
        }
        builder.init().unwrap();

        // Handle args
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());
        let build = !args.flag_no_build;

        {
            // First check that the port is available
            TcpListener::bind(("0.0.0.0", args.flag_port as u16))
                .unwrap_or_else(|err| quit_err!("Unable to listen on port {}: {}", args.flag_port, err));
        }

        args.arg_path.map(|path| helpers::set_cwd(&path));
        if build {
            helpers::build_cwd();
        }
        helpers::serve_cwd(args.flag_port as u16);
    }
}

impl Serve {
    pub fn new(profile: Profile) -> Self {
        if let Some(api) = profile.api_server {
            env::set_var("ALGORITHMIA_API", api);
        }
        env::set_var("ALGORITHMIA_API_KEY", profile.api_key);
        Serve
    }
}

#[cfg(not(target_os = "windows"))]
mod helpers {
    use std::env;
    use std::process::Command;
    use hyper::server::Server;
    use langserver::{LangServer, LangServerMode};

    pub fn set_cwd(path: &str) {
        env::set_current_dir(path)
            .unwrap_or_else(|err| quit_err!("Failed to set working directory: {}", err));
    }

    pub fn build_cwd() {
        let mut child = Command::new("bin/build")
            .spawn()
            .unwrap_or_else(|err| quit_err!("Failed to run `bin/build`: {}", err));
        let _ = child.wait();
    }

    pub fn serve_cwd(port: u16) {
        let langserver = LangServer::start(LangServerMode::Sync, None)
            .unwrap_or_else(|err| quit_err!("Failed to start LangServer: {}", err));

        let _ = Server::http(("0.0.0.0", port))
            .and_then(|s| s.handle(langserver))
            .map(|_listener| {
                println!("Listening on port {}.", port);
            });
    }
}
