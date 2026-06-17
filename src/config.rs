/// Contains everything needed for configuration
use serde::Deserialize;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

/// Json config format
#[derive(Deserialize, Debug)]
struct JsonConfig {
    /// Path of where images will be stored.
    storage_dir: PathBuf,
    /// File where the API key is located.
    api_key_path: PathBuf,
    /// Directory for logs and other misc things
    state_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Config {
    pub storage_dir: PathBuf,
    pub api_key: String,
    pub state_dir: PathBuf,
}

impl Config {
    pub fn load(path: &Path, xdg_dir: BaseDirectories) -> io::Result<Self> {
        let file = File::open(path)?;
        let json: JsonConfig = serde_json::from_reader(file)?;

        let api_key = fs::read_to_string(json.api_key_path)?.trim().to_string();

        let state_dir = json.state_dir.unwrap_or_else(|| {
            xdg_dir
                .state_home
                .unwrap_or_else(|| PathBuf::from("/tmp/apod"))
        });

        if !json.storage_dir.exists() {
            fs::create_dir_all(&json.storage_dir)?;
        }
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir)?;
        }

        let cfg = Config {
            storage_dir: json.storage_dir,
            api_key,
            state_dir,
        };

        Ok(cfg)
    }
}

pub struct OptArgs {
    /// Specify a config file. Default: `$XDG_CONFIG_HOME/apod_desktop/config`
    pub config: Option<PathBuf>,
    /// Prints current background path.
    pub path: bool,
    /// Sets background to random image in `storage_dir`.
    pub random: bool,
}

impl OptArgs {
    /// Will not fail. It is on the user to not make typos.
    pub fn parse() -> Self {
        let mut pargs = pico_args::Arguments::from_env();
        OptArgs {
            config: pargs.opt_value_from_str(["-c", "--config"]).unwrap_or(None),
            path: pargs.contains(["-p", "--path"]),
            random: pargs.contains(["-z", "--random"]),
        }
    }
}
