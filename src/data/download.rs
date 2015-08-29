use super::super::{data, CmdRunner};
use algorithmia::data::{DataFile, DataType, HasDataPath};
use docopt::Docopt;

use std::io;
use std::fs::{self, File};
use std::path::Path;
use std::io::Write;

static USAGE: &'static str = "
Usage: algo download <remote> [<local>]

  Downloads file(s) from the Algorithmia Data API
";

/*
  TODO: Add support for:
    -r                  Recursively download data_uri (if a directory)
    -c <CONCURRENCY>    Number of threads for uploading in parallel [Default: 8]
*/

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: String,
    arg_local: Option<String>,
}

pub struct Download;
impl CmdRunner for Download {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        match args.arg_local {
            Some(l) => Self::download(&*args.arg_remote, &*l),
            None => Self::download(&*args.arg_remote, ".")
        };
    }
}

impl Download {
    fn download(remote_path: &str, local_path: &str) {
        let client = Self::init_client();
        let data_object = &client.clone().data(remote_path);

        match data_object.get_type() {
            Ok(DataType::Dir) => die!("Downloading directories not yet implemented"), //download_dir(data_object.into(), local_path),
            Ok(DataType::File) => download_file(data_object.into(), local_path),
            Err(err) => die!("Error: {:?}", err),
        };
    }
}

fn download_file(data_file: DataFile, local_path: &str) {
    match data_file.get() {
        Ok(mut response) => {
            let mut output: Box<Write> = match local_path {
                "-" => Box::new(io::stdout()),
                p => {
                    let full_path = match fs::metadata(p) {
                        Ok(ref m) if m.is_dir() => Path::new(p).join(data_file.basename().unwrap()),
                        _ => Path::new(p).to_owned(),
                    };
                    match File::create(full_path) {
                        Ok(f) => Box::new(f),
                        Err(err) => die!("Error creating file: {}", err),
                    }
                }
            };

            // Copy downloaded data to the output writer
            match io::copy(&mut response, &mut output) {
                Ok(bytes) => match local_path {
                    "-" => (),
                    _ => println!("{} bytes", data::size_with_suffix(bytes)),
                },
                Err(err) => die!("Error copying data: {}", err),
            }
        },
        Err(e) => die!("Error downloading {}: {:?}", data_file.to_data_uri(), e),
    };
}

// TODO: add concurrency
// fn download_dir(data_dir: DataDir, local_path: &str) {

// }

