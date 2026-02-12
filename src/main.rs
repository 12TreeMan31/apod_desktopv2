// SPDX-License-Identifier: GPL-2.0

use anyhow::Result;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;
use ureq::Agent;

struct OptArgs {
    config_dir: Option<PathBuf>,
    save_mode: bool,
}

#[derive(Deserialize, Debug)]
struct Config {
    api_key: String,
    storage_dir: PathBuf,
    favorite_dir: PathBuf,
    background_path: PathBuf,
}

impl Config {
    fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut config: String = String::new();
        file.read_to_string(&mut config)?;

        Ok(serde_json::from_str(&config)?)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Response {
    copyright: Option<String>,
    date: String,
    explanation: String,
    hdurl: String,
    media_type: String,
    title: String,
}

impl Response {
    fn make_request(agent: Agent, api_key: &str) -> Result<Self> {
        let response: Response = agent
            .get("https://api.nasa.gov/planetary/apod")
            .query("api_key", api_key)
            .call()?
            .body_mut()
            .read_json()?;

        Ok(response)
    }
}

/// Creates a simlink in `favorite_dir` by NEWEST image in `storage_dir` NOT by the current date.
fn save_mode(favorite_dir: &Path, storage_dir: &Path) {
    // All images should be in the format of YYYY-MM-DD and it is assumed ASCII compatible
    let newest_image = fs::read_dir(storage_dir)
        .expect("Failed to read dir")
        .map(|val| val.expect("Failed to read dir").file_name())
        .reduce(|acc, cur| {
            let order = acc.as_encoded_bytes().cmp(cur.as_encoded_bytes());

            if order.is_ge() { acc } else { cur }
        })
        .unwrap();

    let mut original = PathBuf::from(storage_dir);
    original.push(newest_image.clone());

    let mut link = PathBuf::from(favorite_dir);
    link.push(newest_image);

    std::os::unix::fs::symlink(original, link).unwrap();
}

fn main() {
    let mut pargs = pico_args::Arguments::from_env();
    let args = OptArgs {
        config_dir: pargs
            .opt_value_from_str(["-c", "--config"])
            .expect("Invalid Arg"),
        save_mode: pargs.contains(["-s", "--save"]),
    };

    let xdg_dir = xdg::BaseDirectories::with_prefix("apod_desktop");

    // Firsts see if a path was provided then checks XDG for a path if none was found
    let config_path = args.config_dir.unwrap_or_else(|| {
        xdg_dir
            .get_config_file("config")
            .expect("XDG is not configured")
    });

    let Ok(mut config) = Config::load(&config_path) else {
        println!("Unable to read config! Please make sure it exsit and is in valid json.");
        return;
    };

    if args.save_mode {
        save_mode(&config.favorite_dir, &config.storage_dir);
        return;
    }

    env_logger::init();

    let agent_cfg = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent: Agent = agent_cfg.into();

    println!("Read config");
    let Ok(response) = Response::make_request(agent.clone(), &config.api_key) else {
        println!("Unable to get get todays image! Is the api key valid?");
        return;
    };

    if response.media_type != "image" {
        return;
    }

    let mut image_data = agent
        .get(response.hdurl)
        .call()
        .unwrap()
        .body_mut()
        .read_to_vec()
        .unwrap();

    let mut metadata = Metadata::new();
    metadata.set_tag(ExifTag::ImageDescription(response.explanation));
    metadata.set_tag(ExifTag::CreateDate(response.date.clone()));
    metadata
        .write_to_vec(&mut image_data, FileExtension::JPEG)
        .expect("Wont fail");

    let image_name = format!("{}.jpg", response.date);

    config.storage_dir.push(image_name);

    let mut image = File::create(config.storage_dir.clone()).unwrap();
    image.write_all(&image_data).unwrap();
    fs::copy(config.storage_dir, config.background_path).unwrap();
    println!("Saved!")
}
