extern crate algorithmia;
extern crate getopts;

use super::*;
use algorithmia::Service;
use algorithmia::collection::Collection;
use getopts::{Options, Matches};
use std::ascii::AsciiExt;
use std::fs::File;
use std::thread;
use std::process;
use std::sync::{Arc, Semaphore};

static DEFAULT_UPLOAD_CONCURRENCY: u32 = 8;

fn print_usage(opts: &Options, message: Option<&str>) -> ! {
    let brief = r#"
Usage: algo data CMD [CMD_ARGS...]

Supported CMDs,
    SHOW PATH,
    CREATE COLLECTION_PATH,
    DELETE PATH,
    UPLOAD COLLECTION_PATH [-c CONCURRENCY] FILE ...
"#;

    match message {
        Some(msg) => println!("{}\n{}", msg, opts.usage(brief)),
        None => println!("{}", opts.usage(brief)),
    }

    process::exit(1)
}

struct AlgoData {
    service: Service,
}

impl AlgoData {
    fn new() -> AlgoData {
        AlgoData { service: init_service() }
    }

    fn show_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.show() {
            Ok(output) => {
                println!("{}/{} - {} file(s)", output.username, output.collection_name, output.files.len());
                for f in output.files { println!("/{}", f); }
            },
            Err(why) => die(&*format!("ERROR: {:?}", why)),
        };
    }

    fn delete_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.delete() {
            Ok(_) => println!("Deleted collection"),
            Err(why) => die(&*format!("ERROR: {:?}", why)),
        };
    }


    fn create_collection(self, collection: Collection) {
        let my_bucket = self.service.collection(&collection);
        match my_bucket.create() {
            Ok(output) => println!("Created collection: {}/{}", output.username, output.collection_name),
            Err(why) => die(&*format!("ERROR: {:?}", why)),
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
                            Err(e) => die(&*format!("ERROR uploading {}: {:?}", file_path, e)),
                        };
                    },
                    Err(e) => die(&*format!("Failed to open {}: {}", file_path, e)),
                };

                // Release the semaphore
                child_sem.release();
            })
        }).collect();
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}

fn get_concurrency(argopts: &Matches) -> u32 {
    // Get the --concurrency
    match argopts.opt_str("concurrency") {
        Some(nstr) => match nstr.parse::<u32>() {
            Ok(n) => n,
            Err(_) => die(&*format!("Invalid concurrency option: {}", nstr)),
        },
        None => DEFAULT_UPLOAD_CONCURRENCY,
    }
}

pub fn exec_main(mut args: ArgParser) {

    // Get the CMD arg
    let cmd = match args.arg_iter.next() {
        Some(ref arg) => arg.to_ascii_lowercase(),
        None => "show".to_string(),
    };

    // Get the COLLECTIONREF arg
    let col_arg = args.arg_iter.next();
    let collection = match col_arg.as_ref()
            .and_then(|col| { Collection::from_str(&*col).ok() }) {
        Some(col) => col,
        None => print_usage(&args.options, Some("Did not specify valid collection")),
    };


    let data = AlgoData::new();
    match &*cmd {
        "show" => data.show_collection(collection),
        "create" => data.create_collection(collection),
        "delete" => data.delete_collection(collection),
        "upload" => {
            let files: Vec<String> = args.arg_iter.collect();
            data.upload_files(collection, &*files, get_concurrency(&args.matches));
        },
        invalid => {
            println!("Not a valid command: {}", invalid);
            print_usage(&args.options, None);
        }
    }
}
