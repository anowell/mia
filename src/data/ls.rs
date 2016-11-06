use super::super::{data, CmdRunner};
use docopt::Docopt;
use std::cmp;
use algorithmia::Algorithmia;
use algorithmia::data::{DataItem, HasDataPath};
use std::vec::IntoIter;

static USAGE: &'static str = "Usage:
  algo ls [options] [<data-dir>]
  algo dir [options] [<data-dir>]

  List contents of a directory via the Agorithmia Data API

  <data-dir>    Specifies the Algorithmia Data URI
                The 'data://' prefix is optional
                Defaults to 'data://' root path

  Options:
    -l          Use long listing format
";


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_data_dir: Option<String>,
    flag_l: bool,
}

pub struct Ls {
    client: Algorithmia,
}

impl CmdRunner for Ls {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.list_dir(&*args.arg_data_dir.unwrap_or("data://".into()), args.flag_l);
    }
}

impl Ls {
    pub fn new(client: Algorithmia) -> Self {
        Ls { client: client }
    }

    fn list_dir(&self, path: &str, long: bool) {
        let ref c = self.client;
        let my_dir = c.dir(path);

        if long {
            for entry_result in my_dir.list() {
                match entry_result {
                    Ok(DataItem::Dir(d)) => {
                        println!("{:19} {:>5} {}",
                                 "--         --",
                                 "[dir]",
                                 d.basename().unwrap())
                    }
                    Ok(DataItem::File(f)) => {
                        println!("{:19} {:>5} {}",
                                 f.last_modified.format("%Y-%m-%d %H:%M:%S"),
                                 data::size_with_suffix(f.size),
                                 f.basename().unwrap())
                    }
                    Err(err) => die!("Error listing directory: {}", err),
                }
            }
        } else {
            let names: Vec<String> = my_dir.list()
                .map(|entry_result| {
                    match entry_result {
                        Ok(DataItem::Dir(d)) => d.basename().unwrap(),
                        Ok(DataItem::File(f)) => f.basename().unwrap(),
                        Err(err) => die!("Error listing directory: {}", err),
                    }
                })
                .collect();

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
