use super::super::{die, init_service};
use docopt::Docopt;

use std::thread;
use std::sync::{Arc, Semaphore};


static USAGE: &'static str = "
Usage: algo upload [-c CONCURRENCY] <collection> <file>...

    -c <CONCURRENCY>    Number of threads for uploading in parallel [Default: 8]
";

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_upload: bool,
    arg_collection: Option<String>,
    arg_file: Vec<String>,
    flag_c: u32,
}

pub fn print_usage() -> ! { die(USAGE) }

pub fn cmd_main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match args.arg_collection {
        Some(dir) => upload_files(&*dir, args.arg_file, args.flag_c),
        None => print_usage(),
    }
}

fn upload_files(path: &str, file_paths: Vec<String>, concurrency: u32) {
    println!("Uploading {} file(s)...", file_paths.len());
    let service = init_service();
    let path = Arc::new(path);
    let arc_sem = Arc::new(Semaphore::new(concurrency as isize));
    let _: Vec<_> = file_paths.iter().map(|file_path| {
        // Acquire semaphore before we start the thread
        let child_sem = arc_sem.clone();
        child_sem.acquire();
        // println!("Uploading {}", file_path);

        let service_clone = service.clone();
        let path_clone = path.clone();
        thread::scoped( move || {
            let my_bucket = service_clone.collection(&*path_clone);
            let ref bucket = my_bucket;
            match bucket.upload_file(&*file_path) {
                Ok(file_added) => println!("Uploaded {}", file_added.result),
                Err(e) => die(&*format!("ERROR uploading {}: {:?}", file_path, e)),
            };

            // Release the semaphore
            child_sem.release();
        })
    }).collect();
    println!("Finished uploading {} file(s)", file_paths.len())
}