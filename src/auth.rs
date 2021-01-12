use crate::config::{self, Config, Profile};
use crate::{CmdRunner, DynError};
use docopt::Docopt;
use rpassword;
use std::io::{self, BufRead, Write};
use std::vec::IntoIter;
use url::Url;

static USAGE: &'static str = r##"
Usage:
  mia auth [--profile <name>]

  Interactively prompts for authentication credentials. If no profile is specified,
  the changes will apply to the 'default' profile. To use a non-default profile for
  other mia commands, use the --profile <profile> option.

  Profile configuration is stored in $HOME/.algorithmia (Unix/Linux) or
  %LOCALAPPDATA%/Algorithmia (Windows) in the following TOML format:

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
        Auth {
            profile: profile.to_owned(),
        }
    }

    fn prompt_for_auth(profile_name: &str) {
        println!("Configuring authentication for '{}' profile", profile_name);

        // Handle Endpoint URL
        print!(
            "Enter Algorithmia Endpoint [default={}]: ",
            config::DEFAULT_ENDPOINT
        );
        let _ = io::stdout().flush();

        let endpoint = match prompt_for_url() {
            None => Url::parse(config::DEFAULT_ENDPOINT).unwrap(),
            Some(mut u) => {
                // Special handling of 'api.' as it's still likely that many enter the API endpoint
                // instead of the parent domain.
                remove_subdomain(&mut u, "api");
                u
            }
        };

        let api_server = prepend_subdomain(&endpoint, "api");
        let git_server = prepend_subdomain(&endpoint, "git");

        // Handle API Key
        print!("Enter API Key (starts with 'sim'): ");
        let _ = io::stdout().flush();

        let api_key = match rpassword::read_password() {
            Ok(key) => key,
            Err(err) => quit_err!("Cannot read password: {}", err),
        };
        if api_key.len() == 28 && api_key.starts_with("sim") {
            let mut config = Config::read_config().unwrap_or_else(Config::default);
            let profile = Profile::new(api_key.into(), Some(api_server), Some(git_server));

            config.update_profile(profile_name.into(), profile);
            config.write_config();

            if profile_name == "default" {
                println!("Profile is ready to use. Try 'mia ls'");
            } else {
                println!(
                    "Profile is ready to use. Try 'mia ls --profile {}'",
                    profile_name
                );
            }
        } else {
            println!(
                "That API Key doesn't look quite right. No changes made to '{}' profile.",
                profile_name
            );
        }
    }
}

fn prompt_for_url() -> Option<Url> {
    let mut line = String::new();
    let stdin = io::stdin();
    stdin
        .lock()
        .read_line(&mut line)
        .unwrap_or_else(|err| quit_err!("Cannot read input: {}", err));

    if line.trim().is_empty() {
        None
    } else {
        Some(
            parse_url(&line)
                .unwrap_or_else(|err| quit_msg!("Error parsing '{}' as URL: {}", line, err)),
        )
    }
}

fn parse_url(input: &str) -> Result<Url, DynError> {
    let trimmed = input.trim();
    let mut parsed = Url::parse(trimmed)
        .or_else(|err| Url::parse(&format!("https://{}", trimmed)).or(Err(err)))?;
    if !parsed.scheme().starts_with("http") {
        return Err(format!("unsupported scheme '{}'", parsed).into());
    }
    if parsed.host().is_none() {
        return Err(format!("missing host '{}'", parsed).into());
    }

    parsed.set_path("");
    parsed.set_fragment(None);
    Ok(parsed)
}

fn remove_subdomain(input: &mut Url, subdomain: &str) -> bool {
    let host = input.host_str().unwrap();
    let needle = format!("{}.", subdomain);
    if host.starts_with(&needle) {
        let new_host = host[needle.len()..].to_owned();
        input.set_host(Some(&new_host)).unwrap();
        true
    } else {
        false
    }
}

fn prepend_subdomain(input: &Url, subdomain: &str) -> Url {
    let host = input.host_str().unwrap();
    let new_host = format!("{}.{}", subdomain, host);
    let mut output = input.clone();
    output.set_host(Some(&new_host)).unwrap();
    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_url() {
        let assert_url_parse = |input, expected| {
            assert_eq!(parse_url(input).unwrap(), Url::parse(expected).unwrap());
        };

        assert_url_parse("https://algorithmia.com", "https://algorithmia.com");
        assert_url_parse("https://api.algorithmia.com", "https://api.algorithmia.com");
        assert_url_parse("https://methods.example.com", "https://methods.example.com");

        // auto-scheme
        assert_url_parse("methods.example.com", "https://methods.example.com");
        assert_url_parse("api.methods.example.com", "https://api.methods.example.com");

        // ignore path
        assert_url_parse(
            "api.methods.example.com/foo",
            "https://api.methods.example.com",
        );
    }

    #[test]
    fn test_subdomain_removal() {
        let assert_without_api_subdomain = |input, expected| {
            let mut input = Url::parse(input).unwrap();
            let expected = Url::parse(expected).unwrap();
            remove_subdomain(&mut input, "api");
            assert_eq!(input, expected);
        };

        assert_without_api_subdomain("https://api.algorithmia.com", "https://algorithmia.com");
        assert_without_api_subdomain("https://algorithmia.com", "https://algorithmia.com");
        assert_without_api_subdomain("https://methods.example.com", "https://methods.example.com");
        assert_without_api_subdomain(
            "https://api.methods.example.com",
            "https://methods.example.com",
        );
    }

    #[test]
    fn test_subdomain_prepend() {
        let assert_with_subdomain = |subdomain, input, expected| {
            let input = Url::parse(input).unwrap();
            let expected = Url::parse(expected).unwrap();
            let received = prepend_subdomain(&input, subdomain);
            assert_eq!(received, expected);
        };

        assert_with_subdomain(
            "api",
            "https://algorithmia.com",
            "https://api.algorithmia.com",
        );
        assert_with_subdomain(
            "git",
            "https://algorithmia.com",
            "https://git.algorithmia.com",
        );
        assert_with_subdomain(
            "api",
            "https://methods.example.com",
            "https://api.methods.example.com",
        );
        assert_with_subdomain(
            "git",
            "https://methods.example.com",
            "https://git.methods.example.com",
        );
    }
}
