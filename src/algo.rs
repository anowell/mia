use super::*;
use algorithmia::algorithm::Algorithm;
use std::io::Read;
use std::fs::File;
use std::path::Path;

fn read_file_to_string(path: &Path) -> String {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {:?}", display, why),
        Ok(file) => file,
    };

    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Err(why) => panic!("could not read {}: {:?}", display, why),
        Ok(s) => s,
    };
    data
}

pub fn exec_main(mut args: ArgParser) {
    let algoref = match args.arg_iter.next() {
        Some(arg) => arg,
        None => return die("Insufficient arguments")
    };

    let algorithm = match Algorithm::from_str(&*algoref) {
        Ok(algo) => algo,
        Err(_) => return die("Invalid algorithm specification"),
    };

    // Get the --data or --file arg
    let data = match (args.matches.opt_str("data"), args.matches.opt_str("file")) {
        (Some(s), None) => s,
        (None, Some(f)) => read_file_to_string(Path::new(&*f)),
        _ => return die("must specify exactly one of -f or -d"),
    };

    // Instantiate the algorithm service
    let service = init_service();
    let algorithm_service = service.algorithm(&algorithm);

    // Execute the algorithm
    match algorithm_service.exec_raw(&*data) {
        Ok(result) => println!("{}", result),
        Err(e) => die(&*format!("HTTP ERROR: {:?}", e)),
    }
}