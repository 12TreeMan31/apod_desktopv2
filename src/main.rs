// SPDX-License-Identifier: GPL-2.0
use adi_bg::config::{Config, OptArgs};
use adi_bg::response::Response;
use anyhow::{Result, bail};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io, str};
use ureq::Agent;

/// We need to do 3 things.
/// 1. Download todays image, and let that image get set as the background.
/// 2. Get the path of the currently selected image.
/// 3. Method to randomize a set of images.

fn is_dup(img: &OsStr, dir: &Path) -> bool {
    fs::read_dir(dir)
        .expect("Unable to read directory")
        .filter_map(|x| x.ok())
        .any(|s| s.file_name() == img)
}

fn newest_downloaded(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir)
        .expect("Failed to read directory")
        .filter_map(|x| x.ok())
        .map(|x| {
            let bytes = x.file_name().into_encoded_bytes();
            let path = x.path();

            (path, bytes)
        })
        .reduce(|acc, cur| if acc.1 >= cur.1 { acc } else { cur })
        .map(|(path, _)| path)
}

fn download_image(dir: &Path, agent: Agent, res: Response) -> io::Result<PathBuf> {
    let mut img_bytes = res.get_image_bytes(&agent).unwrap();
    res.set_image_metadata(&mut img_bytes)?;

    let mut path: PathBuf = dir.into();
    path.push(res.image_name());

    fs::write(&path, &img_bytes)?;

    Ok(path)
}

fn get_newest_image(config: &Config, agent: Agent) -> Result<PathBuf> {
    let res = Response::make_request(&agent, &config.api_key)?;

    if res.media_type != "image" || is_dup(&res.image_name(), &config.storage_dir) {
        match newest_downloaded(&config.storage_dir) {
            Some(cur_img) => return Ok(cur_img),
            None => unimplemented!(),
        }
    }

    let new_img = download_image(&config.storage_dir, agent, res)?;
    Ok(new_img)
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
            .unwrap_or_else(|| PathBuf::from("config"))
    });

    let Ok(config) = Config::load(&config_path, xdg_dir) else {
        bail!("Unable to read config! Please make sure it exsit and is in valid json.");
    };

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
