use super::super::CmdRunner;
use docopt::Docopt;
use chan;
use std::thread;
use std::sync::{Arc, Mutex};
use std::cmp;


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
    fn upload_files(data_path: &str, file_paths: Vec<String>, max_concurrency: u32) {
        println!("Uploading {} file(s)...", file_paths.len());
        let client = Self::init_client();
        let arc_data_path = Arc::new(data_path.to_string());
        let upload_count = Arc::new(Mutex::new(0));

        let concurrency = cmp::min(file_paths.len(), max_concurrency as usize);

        let rx = {
            let (tx, rx) = chan::sync(concurrency);

            // One Producer thread
            thread::spawn(move || {
                for path in file_paths {
                    tx.send(path);
                }
            });
            // Nested scope causes tx channel to close when the thread spawns complete
            rx
        };


        // Consumers
        let wg = chan::WaitGroup::new();
        for _ in 0..concurrency {
            wg.add(1);
            // Thread data
            let thread_client = client.clone();
            let thread_data_path = arc_data_path.clone();
            let thread_wg = wg.clone();
            let thread_rx = rx.clone();
            let thread_upload_count = upload_count.clone();

            thread::spawn(move || {
                // let c = thread_client.clone();
                for rx_path in thread_rx {
                    let my_dir = thread_client.clone().dir(&*thread_data_path);
                    let ref dir = my_dir;
                    match dir.put_file(&*rx_path) {
                        Ok(file_added) => {
                            println!("Uploaded {}", file_added.result);
                            let mut count = thread_upload_count.lock().unwrap();
                            *count += 1;
                        },
                        Err(e) => die!("Error uploading {}: {}", rx_path, e),
                    };
                }
                thread_wg.done();
            });
        }

        wg.wait();
        println!("Finished uploading {} file(s)", *upload_count.lock().unwrap());
    }
}