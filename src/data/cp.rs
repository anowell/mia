use super::super::CmdRunner;
use algorithmia::Algorithmia;
use algorithmia::data::{DataFile, HasDataPath};
use docopt::Docopt;
use chan;
use std::sync::{Arc, Mutex};
use std::{clone, cmp, io, fs, thread};
use std::fs::File;
use std::path::Path;
use std::vec::IntoIter;
use super::size_with_suffix;

static USAGE: &'static str = "
Usage: algo cp [options] <source>... <dest>

  Copy files to or from the Algorithmia Data API

  An Algorithmia Data URL must be prefixed with  data:// in order to avoid potential path ambiguity

  Options:
   -c <CONCURRENCY>    Number of threads for uploading in parallel [Default: 8]
";

// -r                   Recursive copy if the source is a directory

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_source: Vec<String>,
    arg_dest: String,
    flag_c: u32,
}

pub struct Cp { client: Algorithmia }
impl CmdRunner for Cp {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        let cp_client = CpClient::new(self.client.clone(), args.flag_c, &args.arg_dest);

        // TODO: consider instead checking if "any" source starts_with data://
        match (args.arg_source.iter().any(|s| s.starts_with("data://")), args.arg_dest.starts_with("data://")) {
            (true, false) => cp_client.download(args.arg_source),
            (false, true) => cp_client.upload(args.arg_source),
            (true, true) => die!("Copying directly from Data URI to Data URI is not currently supported"),
            (false, false) => die!("Error: paths potentially ambiguous, prefix remote path with data://"),
        }
    }
}

impl Cp {
    pub fn new(client: Algorithmia) -> Self { Cp{ client:client } }
}

struct CpClient {
    client: Algorithmia,
    max_concurrency: u32,
    dest: Arc<String>,
}

impl clone::Clone for CpClient {
    fn clone(&self) -> CpClient {
        CpClient {
            client: self.client.clone(),
            max_concurrency: self.max_concurrency,
            dest: self.dest.clone(),
        }
    }
}

impl CpClient {
    fn new(client: Algorithmia, max_concurrency: u32, dest: &str) -> CpClient {
        CpClient {
            client: client,
            max_concurrency: max_concurrency,
            dest: Arc::new(dest.to_string()),
        }
    }

    fn upload(&self, sources: Vec<String>) {
        // As long as we aren't recursing, we can be more aggressive in limiting threads we spin up
        // TODO: when supporting dir recursion, fall-back to max_concurrency
        let concurrency = cmp::min(sources.len(), self.max_concurrency as usize);

        let (tx, rx) = chan::sync(self.max_concurrency as usize);
        let wg = chan::WaitGroup::new();
        let completed = Arc::new(Mutex::new(0));

        // One Producer thread queuing up file paths to upload
        thread::spawn(move || {
            for path in sources {
                // TODO: if recursing and is_dir: recurse_and_send(&tx, path)
                tx.send(path);
            }
            drop(tx);
        });


        // Spin up threads to concurrently upload files per that paths received on rx channel
        for _ in 0..concurrency {
            wg.add(1);

            let thread_wg = wg.clone();
            let thread_rx = rx.clone();
            let thread_conn = self.clone();
            let thread_completed = completed.clone();

            thread::spawn(move || {
                for rx_path in thread_rx {
                    let my_dir = thread_conn.client.dir(&*thread_conn.dest);
                    let ref dir = my_dir;
                    match dir.put_file(&*rx_path) {
                        Ok(file_added) => {
                            println!("Uploaded {}", file_added.result);
                            let mut count = thread_completed.lock().unwrap();
                            *count += 1;
                        },
                        Err(e) => die!("Error uploading {}: {}", rx_path, e),
                    };
                }
                thread_wg.done();
            });
        }

        wg.wait();
        println!("Finished uploading {} file(s)", *completed.lock().unwrap());
    }

    fn download(&self, sources: Vec<String>)  {
        // As long as we aren't recursing, we can be more aggressive in limiting threads we spin up
        // TODO: when supporting datadir recursion, fall-back to max_concurrency
        let concurrency = cmp::min(sources.len(), self.max_concurrency as usize);

        let (tx, rx) = chan::sync(self.max_concurrency as usize);
        let wg = chan::WaitGroup::new();
        let completed = Arc::new(Mutex::new(0));

        // One Producer thread queuing up file paths to upload
        thread::spawn(move || {
            for path in sources {
                // TODO: if recursing and is_dir: recurse_remote_and_send(&tx, path)
                tx.send(path);
            }
            drop(tx);
        });


        // Spin up threads to concurrently download files per that paths received on rx channel
        for _ in 0..concurrency {
            wg.add(1);

            let thread_wg = wg.clone();
            let thread_rx = rx.clone();
            let thread_conn = self.clone();
            let thread_completed = completed.clone();

            thread::spawn(move || {
                for rx_path in thread_rx {
                    let my_file = thread_conn.client.file(&*rx_path);
                    match download_file(my_file, &*thread_conn.dest) {
                        Ok(bytes) => {
                            println!("Downloaded {} ({}B)", rx_path, size_with_suffix(bytes));
                            let mut count = thread_completed.lock().unwrap();
                            *count += 1;
                        }
                        Err(err) => die!("{}", err)
                    }
                }
                thread_wg.done();
            });
        }

        wg.wait();
        println!("Finished downloading {} file(s)", *completed.lock().unwrap());
    }
}


fn download_file(data_file: DataFile, local_path: &str) -> Result<u64, String> {
    match data_file.get() {
        Ok(mut response) => {
            let full_path = match fs::metadata(local_path) {
                Ok(ref m) if m.is_dir() => Path::new(local_path).join(data_file.basename().unwrap()),
                _ => Path::new(local_path).to_owned(),
            };

            let mut output = match File::create(full_path) {
                Ok(f) => Box::new(f),
                Err(err) => return Err(format!("Error creating file: {}", err)),
            };

            // Copy downloaded data to the output writer
            match io::copy(&mut response, &mut output) {
                Ok(bytes) => Ok(bytes),
                Err(err) => Err(format!("Error copying data: {}", err)),
            }
        },
        Err(e) => Err(format!("Error downloading ({}): {}", data_file.to_data_uri(), e)),
    }
}

