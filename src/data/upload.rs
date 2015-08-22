use super::super::CmdRunner;
use docopt::Docopt;

use std::thread;
use std::sync::{Arc, Semaphore};


static USAGE: &'static str = "
Usage: algo upload [-c CONCURRENCY] <remote> <local>...

  Uploads file(s) to the Algorithmia Data API

  Options:
   -c <CONCURRENCY>    Number of threads for uploading in parallel [Default: 8]
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_remote: String,
    arg_local: Vec<String>,
    flag_c: u32,
}

pub struct Upload;
impl CmdRunner for Upload {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        Self::upload_files(&*args.arg_remote, args.arg_local, args.flag_c);
    }
}

impl Upload {
    fn upload_files(path: &str, file_paths: Vec<String>, concurrency: u32) {
        println!("Uploading {} file(s)...", file_paths.len());
        let client = Self::init_client();
        let path = Arc::new(path);
        let arc_sem = Arc::new(Semaphore::new(concurrency as isize));
        let _: Vec<_> = file_paths.iter().map(|file_path| {
            // Acquire semaphore before we start the thread
            let child_sem = arc_sem.clone();
            child_sem.acquire();
            // println!("Uploading {}", file_path);

            let client_clone = client.clone();
            let path_clone = path.to_string().clone();
            thread::scoped( move || {
                let my_dir = client_clone.dir(&*path_clone);
                let ref dir = my_dir;
                match dir.put_file(&*file_path) {
                    Ok(file_added) => println!("Uploaded {}", file_added.result),
                    Err(e) => die!("Error uploading {}: {:?}", file_path, e),
                };

                // Release the semaphore
                child_sem.release();
            })
        }).collect();
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}