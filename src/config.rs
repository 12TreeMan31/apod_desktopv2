use anyhow::Result;
use serde::Deserialize;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
struct JsonConfig {
    storage_dir: PathBuf,
    api_key_path: PathBuf,
    favorite_dir: Option<PathBuf>,
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
