use super::super::CmdRunner;
use docopt::Docopt;

use std::thread::{self, JoinHandle};
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
    fn upload_files(data_path: &str, file_paths: Vec<String>, concurrency: u32) {
        println!("Uploading {} file(s)...", file_paths.len());
        let client = Self::init_client();
        let arc_data_path = Arc::new(data_path);
        let arc_sem = Arc::new(Semaphore::new(concurrency as isize));

        let children: Vec<JoinHandle<_>> = file_paths.iter().map( |file_path| {
            // Thread data
            let client_clone = client.clone();
            let data_path_clone = arc_data_path.to_string().clone();
            let file_path_clone = file_path.clone();
            println!("Uploading {}", file_path_clone);

            // Acquire semaphore before we start the thread
            let child_sem = arc_sem.clone();
            child_sem.acquire();

            thread::spawn( move || {
                let my_dir = client_clone.dir(&*data_path_clone);
                let ref dir = my_dir;
                match dir.put_file(&*file_path_clone) {
                    Ok(file_added) => println!("Uploaded {}", file_added.result),
                    Err(e) => die!("Error uploading {}: {:?}", file_path_clone, e),
                };

                // Release the semaphore
                child_sem.release();
            })
        }).collect();

        let _ = children.into_iter().map(|child_thread| { child_thread.join() }).collect::<Vec<_>>();
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}