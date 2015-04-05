#![feature(std_misc)]
#![feature(exit_status)]
#![feature(slice_patterns)]
extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use algorithmia::collection::Collection;
use getopts::Options;
use std::ascii::AsciiExt;
use std::env;
use std::fs::File;
use std::thread;
use std::sync::{Arc, Semaphore};

static DEFAULT_UPLOAD_CONCURRENCY: u32 = 8;

fn print_usage(opts: &Options) {
    let brief = vec![
        "Usage: algodata CMD [CMD_ARGS...]",
        "Supported CMDs",
        "  SHOW COLLECTIONREF",
        "  CREATE COLLECTIONREF",
        "  DELETE COLLECTIONREF",
        "  UPLOAD COLLECTIONREF [-c CONCURRENCY] FILE ..."
    ];
    println!("{}", opts.usage(&*brief.connect("\n")));
    env::set_exit_status(1);
}

struct AlgoData {
    service: Service,
}

impl AlgoData {
    fn new(api_key: &str) -> AlgoData {
        AlgoData { service: Service::new(api_key) }
    }

    fn show_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.show() {
            Ok(output) => {
                println!("{}/{} - {} file(s)", output.username, output.collection_name, output.files.len());
                for f in output.files { println!("/{}", f); }
            },
            Err(why) => {
                println!("ERROR: {:?}", why);
                env::set_exit_status(1);
            },
        };
    }

    fn delete_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.delete() {
            Ok(_) => println!("Deleted collection"),
            Err(why) => {
                println!("ERROR: {:?}", why);
                env::set_exit_status(1);
            },
        };
    }


    fn create_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.create() {
            Ok(output) => println!("Created collection: {}/{}", output.username, output.collection_name),
            Err(why) => {
                println!("ERROR: {:?}", why);
                env::set_exit_status(1);
            },
        };
    }

    fn upload_files(self, collection: Collection, file_paths: &[String], concurrency: u32) {
        println!("Uploading {} file(s)...", file_paths.len());
        let arc_collection = Arc::new(collection);
        let arc_sem = Arc::new(Semaphore::new(concurrency as isize));
        let _: Vec<_> = file_paths.iter().map(|file_path| {
            // Acquire semaphore before we start the thread
            let child_sem = arc_sem.clone();
            child_sem.acquire();
            // println!("Uploading {}", file_path);

            let service = self.service.clone();
            let ts_collection = arc_collection.clone();
            thread::scoped( move || {
                let my_bucket = service.collection(&ts_collection);
                match File::open(file_path) {
                    Ok(mut file) => {
                        let ref bucket = my_bucket;
                        match bucket.upload_file(&mut file) {
                            Ok(file_added) => println!("Uploaded {}", file_added.result),
                            Err(e) => {
                                println!("ERROR uploading {}: {:?}", file_path, e);
                                env::set_exit_status(1);
                            }
                        };
                    },
                    Err(e) => {
                        println!("Failed to open {}: {}", file_path, e);
                        env::set_exit_status(1);
                    }
                };

                // Release the semaphore
                child_sem.release();
            })
        }).collect();
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}


fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("c", "concurrency", &*format!("max concurrent threads to use for uploading files (default = {})", DEFAULT_UPLOAD_CONCURRENCY), "THREADS");

    let argopts = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(f) => {
            println!("Failed to parse args: {}", f);
            print_usage(&opts);
            return;
        }
    };


    let mut args_iter = argopts.free.clone().into_iter().skip(1);
    if argopts.opt_present("help") || args_iter.len() == 0 {
        print_usage(&opts);
        return;
    }

    let api_key = match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            println!("Must set ALGORITHMIA_API_KEY");
            print_usage(&opts);
            return;
        }
    };

    // Get the --concurrency
    let concurrency: u32 = match argopts.opt_str("concurrency") {
        Some(nstr) => {
            match nstr.parse::<u32>() {
                Ok(n) => n,
                Err(_) => {
                    println!("Invalid concurrency option: {}", nstr);
                    print_usage(&opts);
                    return;
                }
            }
        },
        None => DEFAULT_UPLOAD_CONCURRENCY,
    };

    let data = AlgoData::new(&*api_key);

    // Get the CMD arg
    let cmd = match args_iter.next() {
        Some(ref arg) => arg.to_ascii_lowercase(),
        None => "show".to_string(),
    };

    // Get the COLLECTIONREF arg
    let col_arg = args_iter.next();
    let collection = match col_arg.as_ref()
            .and_then(|col| { Collection::from_str(&*col).ok() }) {
        Some(col) => col,
        None => {
            println!("Did not specify valid collection");
            print_usage(&opts);
            return;
        }
    };


    match &*cmd {
        "show" => data.show_collection(collection),
        "create" => data.create_collection(collection),
        "delete" => data.delete_collection(collection),
        "upload" => {
            let files: Vec<String> = args_iter.collect();
            data.upload_files(collection, &*files, concurrency);
        },
        invalid => {
            println!("Not a valid command: {}", invalid);
            print_usage(&opts);
            return;
        }
    }
}
