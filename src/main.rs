// SPDX-License-Identifier: GPL-2.0
use anyhow::Result;
use log::{error, info, warn};
use sid_bg::XDG_NAME;
use sid_bg::config::{Config, OptArgs};
use sid_bg::response::{Query, Response};
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::str;
use std::time::Duration;
use ureq::Agent;

fn is_dup(dir: &[DirEntry], img: &OsStr) -> bool {
    dir.iter().any(|s| s.file_name() == img)
}

/// Returns the absolute path of the newest image in the provided directory.
/// Returns None if the directory is empty
fn newest_downloaded(dir: &[DirEntry]) -> Option<PathBuf> {
    // Use max_by_key?
    dir.iter()
        .map(|x| {
            let bytes = x.file_name().into_encoded_bytes();
            let path = x.path();

            (path, bytes)
        })
        .reduce(|acc, cur| if acc.1 >= cur.1 { acc } else { cur })
        .map(|(path, _)| path)
}

fn get_newest_image(config: &Config, agent: Agent) -> Result<PathBuf> {
    let query = Query::fetch_date(&config.api_key, None);
    info!("{:?}", query);

    let res = Response::make_request(agent.clone(), query)?
        .pop()
        .expect("Won't fail");

    // We would rather not download the image if possible
    let directory: Vec<DirEntry> = fs::read_dir(&config.storage_dir)?
        .filter_map(|x| x.ok())
        .collect();
    if res.media_type() != "image" || is_dup(&directory, &res.image_name()) {
        let Some(path) = newest_downloaded(&directory) else {
            error!("Could not find image. Directory is empty and todays entry was not an image");
            return Err(anyhow::Error::msg("REMOVE"));
        };

        return Ok(path);
    }

    let path = res.download_image(agent, &config.storage_dir)?;
    Ok(path)
}

fn get_bg_name(cache_file: &Path) -> Option<PathBuf> {
    let data = fs::read(cache_file).ok()?;
    let bg_name = str::from_utf8(&data).expect("Invalid utf8");
    Some(PathBuf::from(bg_name))
}

fn set_bg_name(cache_file: &Path, bg_name: &Path) {
    let bg_name = bg_name.to_str().unwrap();
    fs::write(cache_file, bg_name).unwrap();
}

fn main() -> Result<()> {
    let args = OptArgs::parse();
    if args.verbose {
        simple_logger::init_with_level(log::Level::Info).unwrap();
    }

    let xdg_dir = xdg::BaseDirectories::with_prefix(XDG_NAME);

    // Firsts see if a path was provided then checks XDG for a path if none was found
    let config_path = args.config.unwrap_or_else(|| {
        xdg_dir
            .get_config_file("config")
            .unwrap_or_else(|| PathBuf::from("config"))
    });
    log::info!("Config: {:?}", config_path);

    let config = Config::load(&config_path, xdg_dir).unwrap_or_else(|e| {
        error!("Error reading config: {}", e);
        panic!();
    });

    let state_file = config.state_dir.join("bg");

    let background_path: PathBuf = if args.path {
        if let Some(x) = get_bg_name(&state_file) {
            x
        } else {
            unimplemented!()
        }
    } else if args.random {
        unimplemented!()
    } else {
        let agent_cfg = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let agent: Agent = agent_cfg.new_agent();

        let path = get_newest_image(&config, agent)?;

        set_bg_name(&state_file, &path);

        path
    };

    print!("{}", background_path.to_str().unwrap());
    Ok(())
}
