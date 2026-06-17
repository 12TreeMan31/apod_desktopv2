/// Contains everything needed for configuration
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

#[derive(Debug)]
struct RawConfig {
    /// Path to where images will be stored
    storage_dir: Option<PathBuf>,
    /// File to where the API key is located
    api_key_path: Option<PathBuf>,
    /// Directory for logs and other misc things
    state_dir: Option<PathBuf>,
}

impl RawConfig {
    fn new() -> Self {
        RawConfig {
            storage_dir: None,
            api_key_path: None,
            state_dir: None,
        }
    }

    fn parse(cfg_string: String) -> Self {
        let mut cfg = RawConfig::new();

        let pairs = cfg_string
            .lines()
            .filter_map(|x| x.trim().split_once(" "))
            .map(|(s, x)| (s.trim(), x.trim()));

        for (key, val) in pairs {
            match key {
                "storage_dir" => cfg.storage_dir = Some(val.into()),
                "api_key_path" => cfg.api_key_path = Some(val.into()),
                "state_dir" => cfg.state_dir = Some(val.into()),
                _ => (),
            }
        }

        cfg
    }
    fn verify(self) -> Option<Self> {
        if self.api_key_path == None || self.storage_dir == None {
            return None;
        }

        Some(self)
    }
}

#[derive(Debug)]
pub struct Config {
    pub storage_dir: PathBuf,
    pub api_key: String,
    pub state_dir: PathBuf,
}

impl Config {
    pub fn load(path: &Path, xdg_dir: BaseDirectories) -> io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let Some(rcfg) = RawConfig::parse(raw).verify() else {
            unimplemented!()
        };

        let Some(api_key_path) = rcfg.api_key_path else {
            unreachable!()
        };
        let Some(storage_dir) = rcfg.storage_dir else {
            unreachable!()
        };

        let api_key = fs::read_to_string(&api_key_path)?.trim().to_string();

        let state_dir = rcfg.state_dir.unwrap_or_else(|| {
            xdg_dir
                .state_home
                .unwrap_or_else(|| PathBuf::from("/tmp/apod"))
        });

        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir)?;
        }

        let cfg = Config {
            storage_dir,
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
