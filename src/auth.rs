use super::{CmdRunner, get_config_path};
use docopt::Docopt;
use std::fs::File;
use std::io::{self, Read, Write};
use toml::{self, Parser, Table, Value};

static USAGE: &'static str = r#"
Usage:
  algo auth [<profile>]

  Interactively prompts for authentication credentials. If no profile is specified,
  the changes will apply to the 'default' profile. To use a non-default profile for
  other algo commands, use the --profile <profile> option.

  Profile configuration is stored in $HOME/.algorithmia (Unix/Linux) or
  %LOCALAPPDATA%/algorithmia (Windows) in the following TOML format:

    [profiles]

    [profiles.default]
    sim_key = "sim1234567890abcdef"
"#;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_profile: Option<String>,
}

pub struct Auth { _priv: () }
impl CmdRunner for Auth {
    fn get_usage() -> &'static str { USAGE }

    fn cmd_main(&self) {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        Auth::prompt_for_auth(&args.arg_profile.unwrap_or("default".into()));
    }
}


impl Auth {
    pub fn new() -> Self { Auth{ _priv: () } }

    fn prompt_for_auth(profile_name: &str) {
        println!("Configuring authentication for '{}' profile", profile_name);
        print!("Enter API Key (prefixed with 'sim'): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);

        let api_key = input.trim();
        if api_key.len() == 28 && api_key.starts_with("sim") {
            let mut config = Self::read_config().unwrap_or(Table::new());
            let profile = Self::make_profile(api_key.into());

            Self::update_profile(&mut config, profile_name.into(), profile);
            Self::write_config(config);

            match profile_name {
                "default" => println!("Profile is ready to use. Test with 'auth ls'"),
                p => println!("Profile is ready to use. Test with 'auth ls --profile {}", p),
            };
        } else {
            println!("That API Key doesn't look quite right. No changes made to '{}' profile.", profile_name);
        }

    }

    fn make_profile(api_key: String) -> Table {
        let mut profile = Table::new();
        profile.insert("simple_key".into(), Value::String(api_key));
        profile
    }

    pub fn read_profile(profile_name: String) -> Option<Table> {
        match Self::read_config() {
            Some(t) => match Value::Table(t).lookup(&format!("profiles.{}", profile_name)) {
                Some(&Value::Table(ref p)) => Some(p.clone()),
                Some(_) => die!("Invalid profile format in {}", get_config_path()),
                None => None,
            },
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
                    die!("Unable to parse {}. Consider deleting and re-running 'algo auth'", conf_path);
                });
                Some(config)
            },
            Err(_) => None,
        }
    }

    fn write_config(config: Table) {
        let conf_path = get_config_path();

        let output = toml::encode_str(&Value::Table(config));

        let _ = match File::create(&conf_path) {
            Ok(mut f) => f.write_all(output.as_bytes()),
            Err(e) => die!("Unable to write config file: {}", e),
        };

        // TODO: ACL 0600 on Linux/Unix
    }

    fn update_profile(config: &mut Table, name: String, value: Table) {
        if config.contains_key("profiles") {
            if let Some(&mut Value::Table(ref mut section)) = config.get_mut("profiles") {
                section.remove(&name);
                section.insert(name, Value::Table(value));
            } else {
                die!("Unable to parse [profiles] section of configuration");
            }
        } else {
            let mut section = Table::new();
            section.insert(name, Value::Table(value));
            config.insert("profiles".into(), Value::Table(section));

        }
    }

}