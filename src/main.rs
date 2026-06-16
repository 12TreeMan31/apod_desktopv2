// SPDX-License-Identifier: GPL-2.0
use adi_bg::config::{Config, OptArgs};
use adi_bg::response::Response;
use anyhow::{Result, bail};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str;
use std::time::Duration;
use ureq::Agent;

/// We need to do 3 things.
/// 1. Download todays image, and let that image get set as the background.
/// 2. Get the path of the currently selected image.
/// 3. Method to randomize a set of images.

fn fetch_image(config: &Config) -> Option<PathBuf> {
    let agent_cfg = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent: Agent = agent_cfg.new_agent();

    let Ok(res) = Response::make_request(&agent, &config.api_key) else {
        println!("Couldn't make request");
        return None;
    };

    if res.media_type != "image" {
        println!("Is not an image");
        return None;
    }

    // Todays image name
    let img_name = res.image_name();

    // Checks to see if image is already downloaded
    let is_new = fs::read_dir(&config.storage_dir)
        .expect("Isn't new")
        .filter_map(|x| x.ok())
        .all(|s| s.file_name() != img_name);

    if !is_new {
        return None;
    }

    // Save image
    let mut img_bytes = res.get_image_bytes(&agent).unwrap();
    res.set_image_metadata(&mut img_bytes);

    let mut path = config.storage_dir.clone();
    path.push(img_name);

    let mut image = File::create(&path).ok()?;
    image.write_all(&img_bytes);

    Some(path)
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
    let xdg_dir = xdg::BaseDirectories::with_prefix("adi-bg");

    // Firsts see if a path was provided then checks XDG for a path if none was found
    let config_path = args.config.unwrap_or_else(|| {
        xdg_dir
            .get_config_file("config")
            .expect("XDG is not configured")
    });

    let Ok(config) = Config::load(&config_path) else {
        bail!("Unable to read config! Please make sure it exsit and is in valid json.");
    };

    let mut cache_file = config
        .state_dir
        .clone()
        .unwrap_or_else(|| xdg_dir.state_home.expect("XDG is not configured"));
    cache_file.push("bg");

    let background_path: PathBuf = if args.path {
        if let Some(x) = get_bg_name(&cache_file) {
            x
        } else {
            unimplemented!()
        }
    } else if args.random {
        unimplemented!()
    } else {
        let path = fetch_image(&config).unwrap();
        set_bg_name(&cache_file, &path);

        path
    };

    print!("{}", background_path.to_str().unwrap());
    Ok(())
}
