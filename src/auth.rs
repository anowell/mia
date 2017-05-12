use CmdRunner;
use config::{self, Config, Profile};
use docopt::Docopt;
use std::io::{self, Write, BufRead};
use std::vec::IntoIter;
use rpassword;
use url::Url;

static USAGE: &'static str = r##"
Usage:
  algo auth [--profile <name>]

  Interactively prompts for authentication credentials. If no profile is specified,
  the changes will apply to the 'default' profile. To use a non-default profile for
  other algo commands, use the --profile <profile> option.

  Profile configuration is stored in $HOME/.algorithmia (Unix/Linux) or
  %LOCALAPPDATA%/algorithmia (Windows) in the following TOML format:

    [profiles]

    [profiles.default]
    sim_key = "sim1234567890abcdef"
"##;

#[derive(RustcDecodable, Debug)]
struct Args {
    // commented out because profile is stripped by `main` and passed directly into `new`
    // arg_profile: Option<String>,
}

pub struct Auth {
    profile: String,
}
impl CmdRunner for Auth {
    fn get_usage() -> &'static str {
        USAGE
    }

    fn cmd_main(&self, argv: IntoIter<String>) {
        let _args: Args = Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());

        Auth::prompt_for_auth(&self.profile);
    }
}



impl Auth {
    pub fn new(profile: &str) -> Self {
        Auth { profile: profile.to_owned() }
    }

    fn prompt_for_auth(profile_name: &str) {
        println!("Configuring authentication for '{}' profile", profile_name);

        // Handle Endpoint URL
        print!("Enter API Endpoint [{}]: ", config::DEFAULT_API_SERVER);
        let _ = io::stdout().flush();
        let api_server = prompt_for_url();


        // Handle Git URL
        let git_server = match api_server {
            Some(ref api_server) if api_server.as_str() != config::DEFAULT_API_SERVER => {
                let default_git = api_server.as_str().replace("//api.", "//git.");
                print!("Enter Git Endpoint [{}]: ", &default_git);
                let _ = io::stdout().flush();
                Some(prompt_for_url().unwrap_or_else(|| Url::parse(&default_git).unwrap()))
            }
            _ => None
        };

        // Handle API Key
        print!("Enter API Key (prefixed with 'sim'): ");
        let _ = io::stdout().flush();

        let api_key = match rpassword::read_password() {
            Ok(key) => key,
            Err(err) => quit_err!("Cannot read password: {}", err),
        };
        if api_key.len() == 28 && api_key.starts_with("sim") {
            let mut config = Config::read_config().unwrap_or_else(Config::default);
            let profile = Profile::new(api_key.into(), api_server, git_server);

            config.update_profile(profile_name.into(), profile);
            config.write_config();

            match profile_name {
                "default" => println!("Profile is ready to use. Test with 'algo ls'"),
                p => println!("Profile is ready to use. Test with 'algo ls --profile {}'", p)
            };
        } else {
            println!("That API Key doesn't look quite right. No changes made to '{}' profile.",
                     profile_name);
        }

    }

}

fn prompt_for_url() -> Option<Url> {
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line)
        .unwrap_or_else(|err| quit_err!("Cannot read input: {}", err));

    if line.trim().is_empty() {
        None
    } else {
        let trimmed = line.trim();
        let parsed = Url::parse(trimmed).unwrap_or_else(|err|
            Url::parse(&format!("https://{}", trimmed)).unwrap_or_else(|_|
                quit_err!("Cannot parse '{}' as URL: {}", trimmed, err)
            )
        );
        if !parsed.scheme().starts_with("http") {
            quit_msg!("Invalid URL: '{}'", parsed);
        }

        Some(parsed)
    }

}
