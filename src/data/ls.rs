use super::super::{data, CmdRunner};
use docopt::Docopt;
use std::cmp;

static USAGE: &'static str = "
Usage:
  algo ls [options] [<remote>]

  List contents of a directory via the Agorithmia Data API

  <remote>      Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
                Defaults to 'data://' root path

  Options:
    -l          Use long listing format
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: Option<String>,
    flag_l: bool,
}

pub struct Ls;
impl CmdRunner for Ls {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        Self::list_dir(&*args.arg_remote.unwrap_or("data://".into()), args.flag_l);
    }
}

impl Ls {
    fn list_dir(path: &str, long: bool) {
        let my_dir = Self::init_client().dir(path);
        match my_dir.show() {
            Ok(output) => {
                let files = output.files.unwrap_or(vec![]);
                let folders = output.folders.unwrap_or(vec![]);

                // TODO: colorize dirs if tty
                if long {
                    for f in folders { println!("{:19} {:>5} {}", "--         --", "[dir]", f.name); }
                    for f in files { println!("{:19} {:>5} {}", f.last_modified.format("%Y-%m-%d %H:%M:%S"), data::size_with_suffix(f.size), f.filename); }
                } else {
                    let width = 80; // TODO: get_winsize()

                    let col_width = 2 + cmp::max(
                        files.iter().fold(0, |max, f| cmp::max(max, f.filename.len())),
                        folders.iter().fold(0, |max, f| cmp::max(max, f.name.len())),
                    );

                    let mut offset = 0;
                    let mut print_col = |msg: &str| {
                        if offset + col_width > width {
                            println!("");
                            offset = 0;
                        }
                        print!("{:1$}", msg, col_width);
                        offset = offset + col_width;
                    };

                    for f in folders { print_col(&*f.name); }
                    for f in files { print_col(&*f.filename); }
                    println!("");
                }
            },
            Err(why) => die!("ERROR: {:?}", why),
        };
    }
}
