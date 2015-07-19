use super::super::CmdRunner;
use docopt::Docopt;

static USAGE: &'static str = "
Usage:
  algo ls <collection>
";
/*
    TODO: add -l flag
*/

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_collection: Option<String>,
}

pub struct Ls;
impl CmdRunner for Ls {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        match args.arg_collection {
            Some(dir) => Self::list_dir(&*dir),
            None => Self::print_usage(),
        };
    }
}

impl Ls {
    fn list_dir(path: &str) {
        let my_dir = Self::init_client().dir(path);
        match my_dir.show() {
            Ok(output) => {
                let files = output.files.unwrap_or(vec![]);
                println!("{} - {} file(s)", my_dir.to_data_uri(), files.len());
                for f in files { println!("/{}", f.filename); }
            },
            Err(why) => die!("ERROR: {:?}", why),
        };
    }
}
