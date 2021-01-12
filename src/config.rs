use algorithmia::Algorithmia;
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use toml;
use url::Url;

#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

pub static DEFAULT_ENDPOINT: &str = "https://algorithmia.com";
pub static DEFAULT_API_SERVER: &str = "https://api.algorithmia.com";
pub static DEFAULT_GIT_SERVER: &str = "https://git.algorithmia.com";

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    profiles: BTreeMap<String, Profile>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Profile {
    api_server: Option<String>,
    git_server: Option<String>,
    api_key: String,
}

impl Profile {
    pub fn new(api_key: String, api_server: Option<Url>, git_server: Option<Url>) -> Profile {
        Profile {
            api_server: api_server.map(|s| s.as_str().trim_end_matches('/').to_owned()),
            git_server: git_server.map(|s| s.as_str().trim_end_matches('/').to_owned()),
            api_key: api_key,
        }
    }

    pub fn client(&self) -> Algorithmia {
        match self.api_server {
            Some(ref api) => Algorithmia::client_with_url(&*self.api_key, api).unwrap(),
            None => Algorithmia::client(&*self.api_key).unwrap(),
        }
    }

    pub fn api_server(&self) -> &str {
        self.api_server
            .as_ref()
            .map(String::as_ref)
            .unwrap_or(DEFAULT_API_SERVER)
    }

    pub fn git_server(&self) -> String {
        self.git_server.clone().unwrap_or_else(|| {
            let needle = "//api";
            match &self.api_server {
                Some(api_server) if api_server.contains(needle) => {
                    api_server.replace("//api.", "//git.")
                }
                _ => DEFAULT_GIT_SERVER.to_string(),
            }
        })
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

impl Config {
    pub fn read_config() -> Option<Config> {
        let conf_path = get_config_path();

        match File::open(&conf_path) {
            Ok(mut f) => {
                let mut conf_toml = String::new();
                let _ = f.read_to_string(&mut conf_toml);
                let config = toml::from_str(&conf_toml).unwrap_or_else(|err| {
                    quit_msg!(
                        "Unable to parse {}: {}\nConsider deleting and re-running 'mia auth'",
                        conf_path.display(),
                        err
                    );
                });
                Some(config)
            }
            Err(_) => None,
        }
    }

    pub fn update_profile(&mut self, name: String, value: Profile) {
        self.profiles.insert(name, value);
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    pub fn write_config(&mut self) {
        let output = toml::to_string(&self).unwrap();

        let _ = match open_writable_config() {
            Ok(mut f) => f.write_all(output.as_bytes()),
            Err(e) => quit_err!("Unable to write config file: {}", e),
        };
    }
}

impl Profile {
    pub fn lookup(profile: &str) -> Profile {
        Config::read_config()
            .and_then(|c| c.get_profile(profile).cloned())
            .unwrap_or_else(|| quit_msg!("{} profile not found. Run 'mia auth {0}'", profile))
    }
}

pub fn get_config_path() -> PathBuf {
    let app_dir = if cfg!(windows) {
        PathBuf::from(format!("{}/Algorithmia", env::var("LOCALAPPDATA").unwrap()))
    } else {
        PathBuf::from(format!("{}/.algorithmia", env::var("HOME").unwrap()))
    };

    if !app_dir.is_dir() {
        fs::create_dir(&app_dir).unwrap_or_else(|err| {
            quit_err!("Failed to create app dir '{}': {}", app_dir.display(), err)
        });
    }
    app_dir.join("config")
}

#[cfg(not(unix))]
fn open_writable_config() -> Result<File, io::Error> {
    let conf_path = get_config_path();
    File::create(&conf_path)
}

#[cfg(unix)]
fn open_writable_config() -> Result<File, io::Error> {
    let conf_path = get_config_path();
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&conf_path)
}
