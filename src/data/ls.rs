use super::super::{data, CmdRunner};
use docopt::Docopt;
use std::cmp;
use algorithmia::data::{DirEntry, HasDataPath};

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

        if long {
            for entry_result in my_dir.list() {
                    match entry_result {
                        Ok(DirEntry::Dir(d)) => println!("{:19} {:>5} {}", "--         --", "[dir]", d.basename().unwrap()),
                        Ok(DirEntry::File(f)) => println!("{:19} {:>5} {}", f.last_modified.format("%Y-%m-%d %H:%M:%S"), data::size_with_suffix(f.size), f.basename().unwrap()),
                        Err(err) => die!("ERROR: {:?}", err),
                    }
            }
        } else {
            let names: Vec<String> = my_dir.list().map(|entry_result| {
                match entry_result {
                    Ok(DirEntry::Dir(d)) => d.basename().unwrap(),
                    Ok(DirEntry::File(f)) => f.basename().unwrap(),
                    Err(err) => die!("ERROR: {:?}", err),
                }
            }).collect();

            let width = 80; // TODO: get_winsize()

            let col_width = 2 + names.iter().fold(0, |max, name| cmp::max(max, name.len()));

            let mut offset = 0;
            for name in names {
                if offset + col_width > width {
                    println!("");
                    offset = 0;
                }
                print!("{:1$}", name, col_width);
                offset = offset + col_width;
            }

            println!("");
        }
    }
}
