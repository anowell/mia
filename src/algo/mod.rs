pub use self::run::Run;
pub use self::runlocal::RunLocal;
pub use self::clone::GitClone;
pub use self::serve::Serve;

mod run;
mod runlocal;
mod clone;
mod serve;

use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;
use rustc_serialize::json::Json;

#[derive(Debug)]
enum InputData {
    Text(String),
    Json(String),
    Binary(Vec<u8>),
}

impl InputData {

    // Auto-detect the InputData type
    // 1. Json if it parses as JSON
    // 2. Text if it parses as UTF-8
    // 3. Fallback to binary
    fn auto(reader: &mut Read) -> InputData {
        let mut bytes: Vec<u8> = Vec::new();
        if let Err(err) = reader.read_to_end(&mut bytes) {
            die!("Read error: {}", err);
        }

        match String::from_utf8(bytes) {
            Ok(data) => match Json::from_str(&data) {
                Ok(_) => InputData::Json(data),
                Err(_) => InputData::Text(data),
            },
            Err(not_utf8) => InputData::Binary(not_utf8.into_bytes()),
        }
    }

    fn text(reader: &mut Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Text(data),
            Err(err) => die!("Read error: {}", err),
        }
    }

    fn json(reader: &mut Read) -> InputData {
        let mut data = String::new();
        match reader.read_to_string(&mut data) {
            Ok(_) => InputData::Json(data),
            Err(err) => die!("Read error: {}", err),
        }
    }

    fn binary(reader: &mut Read) -> InputData {
        let mut bytes: Vec<u8> = Vec::new();
        match reader.read_to_end(&mut bytes) {
            Ok(_) => InputData::Binary(bytes),
            Err(err) => die!("Read error: {}", err),
        }
    }
}


// The device specified by --output flag
// Only the result or response is written to this device
struct OutputDevice {
    writer: Box<Write>
}

impl OutputDevice {
    fn new(output_dest: &Option<String>) -> OutputDevice {
        match output_dest {
            &Some(ref file_path) => match File::create(file_path) {
                Ok(buf) => OutputDevice{ writer: Box::new(buf) },
                Err(err) => die!("Unable to create file: {}", err),
            },
            &None => OutputDevice{ writer: Box::new(io::stdout()) },
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        match self.writer.write(bytes) {
            Ok(_) => (),
            Err(err) => die!("Error writing output: {}", err),
        }
    }

    fn writeln(&mut self, bytes: &[u8]) {
        self.write(bytes);
        self.write(b"\n");
    }
}

fn get_src(src: &str) -> Box<Read> {
    match src {
        "-" => Box::new(io::stdin()) as Box<Read>,
        s => open_file(Path::new(&s)),
    }
}

fn open_file(path: &Path) -> Box<Read> {
    let display = path.display();
    let file = match File::open(&path) {
        Err(err) => die!("Error opening {}: {}", display, err),
        Ok(file) => file,
    };
    Box::new(file)
}
