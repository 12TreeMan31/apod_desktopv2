/// Contains everything needed for configuration
use anyhow::Result;
use serde::Deserialize;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// Json config format
#[derive(Deserialize, Debug)]
struct JsonConfig {
    /// Path of where images will be stored.
    storage_dir: PathBuf,
    /// File where the API key is located.
    api_key_path: PathBuf,
    /// Path of where to keep favorited images; defaults to `storage_dir/liked/`.
    favorite_dir: Option<PathBuf>,
    /// To be removed
    background_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Config {
    pub storage_dir: PathBuf,
    pub api_key: String,
    pub favorite_dir: PathBuf,
    pub background_path: PathBuf,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let json: JsonConfig = serde_json::from_reader(file)?;

        let fav_dir = match json.favorite_dir {
            Some(x) => x,
            None => json.storage_dir.join("/liked/"),
        };

        let background = match json.background_path {
            Some(x) => x,
            None => json.storage_dir.clone(),
        };

        let api_key = fs::read_to_string(json.api_key_path)?;

        let cfg = Config {
            storage_dir: json.storage_dir,
            api_key,
            favorite_dir: fav_dir,
            background_path: background,
        };

        Ok(cfg)
    }
}

pub struct OptArgs {
    /// Specify a config file. Default: `$XDG_CONFIG_HOME/apod_desktop/config`
    pub config: Option<PathBuf>,
    /// Favorite NEWEST image in `storage_dir`
    pub save: bool,
    /// Show help message and quit.
    help: bool,
}

impl OptArgs {
    pub fn parse() -> Self {
        let mut pargs = pico_args::Arguments::from_env();
        OptArgs {
            config: pargs.opt_value_from_str(["-c", "--config"]).unwrap_or(None),
            save: pargs.contains(["-s", "--save"]),
            help: pargs.contains(["-h", "--help"]),
        }
    }
}
