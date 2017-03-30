
extern crate rpassword;

use super::{CmdRunner, get_config_path};
use docopt::Docopt;
use std::fs::File;
use std::io::{self, Read, Write, BufRead};
use std::vec::IntoIter;
use toml::{self, Parser, Table, Value};
use url::Url;

#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

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
        print!("Enter API Endpoint [https://api.algorithmia.com]: ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line)
            .unwrap_or_else(|err| quit_err!("Cannot read API endpoint: {}", err));
        let api_server = if line.trim().is_empty() {
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
        };

        // Handle API Key
        print!("Enter API Key (prefixed with 'sim'): ");
        let _ = io::stdout().flush();

        let api_key = match rpassword::read_password() {
            Ok(key) => key,
            Err(err) => quit_err!("Cannot read password: {}", err),
        };
        if api_key.len() == 28 && api_key.starts_with("sim") {
            let mut config = Self::read_config().unwrap_or_else(Table::new);
            let profile = Self::make_profile(api_key.into(), api_server);

            Self::update_profile(&mut config, profile_name.into(), profile);
            Self::write_config(config);

            match profile_name {
                "default" => println!("Profile is ready to use. Test with 'algo ls'"),
                p => println!("Profile is ready to use. Test with 'algo ls --profile {}'", p)
            };
        } else {
            println!("That API Key doesn't look quite right. No changes made to '{}' profile.",
                     profile_name);
        }

    }

    fn make_profile(api_key: String, api_server: Option<Url>) -> Table {
        let mut profile = Table::new();
        api_server.map(|s|
            profile.insert(
                "api_server".to_owned(),
                Value::String(s.as_str().trim_right_matches('/').to_owned()),
            )
        );
        profile.insert("api_key".to_owned(), Value::String(api_key));
        profile
    }

    pub fn read_profile(profile_name: String) -> Option<Table> {
        match Self::read_config() {
            Some(t) => {
                match Value::Table(t).lookup(&format!("profiles.{}", profile_name)) {
                    Some(&Value::Table(ref p)) => Some(p.clone()),
                    Some(_) => quit_msg!("Invalid profile format in {}", get_config_path().display()),
                    None => None,
                }
            }
            None => None,
        }
    }

    fn read_config() -> Option<Table> {
        let conf_path = get_config_path();

        match File::open(&conf_path) {
            Ok(mut f) => {
                let mut conf_toml = String::new();
                let _ = f.read_to_string(&mut conf_toml);
                let config = Parser::new(&conf_toml).parse().unwrap_or_else(|| {
                    quit_msg!("Unable to parse {}. Consider deleting and re-running 'algo auth'",
                         conf_path.display());
                });
                Some(config)
            }
            Err(_) => None,
        }
    }

    fn write_config(config: Table) {
        let output = toml::encode_str(&Value::Table(config));

        let _ = match open_writable_config() {
            Ok(mut f) => f.write_all(output.as_bytes()),
            Err(e) => quit_err!("Unable to write config file: {}", e),
        };
    }

    fn update_profile(config: &mut Table, name: String, value: Table) {
        if config.contains_key("profiles") {
            if let Some(&mut Value::Table(ref mut section)) = config.get_mut("profiles") {
                section.remove(&name);
                section.insert(name, Value::Table(value));
            } else {
                quit_msg!("Unable to parse [profiles] section of configuration");
            }
        } else {
            let mut section = Table::new();
            section.insert(name, Value::Table(value));
            config.insert("profiles".into(), Value::Table(section));

        }
    }
}

#[cfg(not(unix))]
fn open_writable_config() -> Result<File, io::Error> {
    let conf_path = get_config_path();
    File::create(&conf_path)
}

#[cfg(unix)]
fn open_writable_config() -> Result<File, io::Error> {
    let conf_path = get_config_path();
    OpenOptions::new().create(true).truncate(true).write(true).mode(0o600).open(&conf_path)
}
