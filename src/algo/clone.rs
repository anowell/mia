use super::super::CmdRunner;
use docopt::Docopt;
use config::Profile;

use std::process::Command;
use std::vec::IntoIter;

static USAGE: &'static str = r##"Usage:
  algo clone <algorithm> [<directory>]

  <algorithm> syntax: USERNAME/ALGONAME
  \
     Recommend specifying a version since algorithm costs can change between minor versions.

  \
     This command is basically a wrapper for:
    git clone \
     https://git.algorithmia.com/git/USERNAME/ALGONAME.git

  Examples:
    algo clone \
     anowell/bcrypt                         Clones an algorithm repo
    algo clone anowell/Pinky \
     pinky-quotes             Clones an algorithm repo into a specific directory
"##;


#[derive(RustcDecodable, Debug)]
struct Args {
    arg_algorithm: String,
    arg_directory: Option<String>,
}

pub struct GitClone { profile: Profile }
impl CmdRunner for GitClone {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        self.git_clone(&*args.arg_algorithm,
                       args.arg_directory.as_ref().map(String::as_ref));
    }
}

impl GitClone {
    pub fn new(profile: Profile) -> Self {
        GitClone{ profile }
    }

    fn git_clone(&self, algo: &str, dir_opt: Option<&str>) {
        let url = format!("{}/git/{}.git", self.profile.git_server(), algo);
        println!("Cloning {}", &url);

        let mut cmd = Command::new("git");
        cmd.arg("clone").arg(&url);

        if let Some(dir) = dir_opt {
            cmd.arg(dir);
        }

        let mut child = cmd.spawn()
            .unwrap_or_else(|_| quit_msg!("Failed to `git clone`. Is git installed?"));
        let _ = child.wait();

    }
}
