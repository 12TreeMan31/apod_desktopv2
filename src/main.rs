// SPDX-License-Identifier: GPL-2.0

use anyhow::{Result, bail};
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
    config_file: Option<PathBuf>,
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
        let file = File::open(path)?;
        let cfg: Self = serde_json::from_reader(file)?;
        Ok(cfg)
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
    fn make_request(agent: &Agent, api_key: &str) -> Result<Self> {
        let mut response = agent
            .get("https://api.nasa.gov/planetary/apod")
            .query("api_key", api_key)
            .call()?;

        if response.status() != 200 {
            bail!(
                "Unable to get todays information,g ot status {}",
                response.status()
            );
        }

        let json: Response = response.body_mut().read_json()?;
        Ok(json)
    }

    /// Gets todays image located in `hdurl`. It will then populate the EXIF Tags with the
    /// image information.
    ///
    /// `Response::make_request` should have been called before this.
    fn get_image_data(&self, agent: &Agent) -> Result<Vec<u8>> {
        let mut image_data = agent.get(&self.hdurl).call()?;

        if image_data.status() != 200 {
            bail!(
                "Unable to get image data, got status {}",
                image_data.status()
            )
        }

        let mut image_data = image_data.body_mut().read_to_vec()?;
        let mut metadata = Metadata::new();
        metadata.set_tag(ExifTag::ImageDescription(self.explanation.clone()));
        metadata.set_tag(ExifTag::CreateDate(self.date.clone()));

        if let Some(copyright) = self.copyright.as_ref() {
            metadata.set_tag(ExifTag::Copyright(copyright.clone()));
        }

        metadata
            .write_to_vec(&mut image_data, FileExtension::JPEG)
            .expect("Wont fail");

        Ok(image_data)
    }
}

fn get_file_extension(url: &str) -> &str {
    let (_, extension) = url
        .rsplit_once(".")
        .expect("somehow got a url without a file extension");

    extension
}

/// Creates a simlink in `favorite_dir` by NEWEST image in `storage_dir` NOT by the current date.
/// The reason for this is that APOD doesn't get updated at midnight in whatever timezone you're in.
fn save_mode(favorite_dir: &Path, storage_dir: &Path) -> Result<()> {
    // All images should be in the format of YYYY-MM-DD and it is assumed ASCII compatible
    let newest_image = fs::read_dir(storage_dir)
        .expect("Failed to read dir")
        .map(|val| val.unwrap().file_name())
        .reduce(|acc, cur| {
            let order = acc.as_encoded_bytes().cmp(cur.as_encoded_bytes());

            if order.is_ge() { acc } else { cur }
        })
        .unwrap();

    let mut original = PathBuf::from(storage_dir);
    original.push(newest_image.clone());

    let mut link = PathBuf::from(favorite_dir);
    link.push(newest_image);

    std::os::unix::fs::symlink(original, link)?;

    Ok(())
}

fn main() -> Result<()> {
    let mut pargs = pico_args::Arguments::from_env();
    let args = OptArgs {
        config_file: pargs.opt_value_from_str(["-c", "--config"]).unwrap_or(None),
        save_mode: pargs.contains(["-s", "--save"]),
    };

    let xdg_dir = xdg::BaseDirectories::with_prefix("apod_desktop");

    // Firsts see if a path was provided then checks XDG for a path if none was found
    let config_path = args.config_file.unwrap_or_else(|| {
        xdg_dir
            .get_config_file("config")
            .expect("XDG is not configured")
    });

    let Ok(mut config) = Config::load(&config_path) else {
        bail!("Unable to read config! Please make sure it exsit and is in valid json.");
    };

    if args.save_mode {
        return save_mode(&config.favorite_dir, &config.storage_dir);
    }

    let agent_cfg = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    let agent: Agent = agent_cfg.into();

    println!("Read config");
    let Ok(response) = Response::make_request(&agent, &config.api_key) else {
        bail!("Unable to get get todays image! Is the api key valid?");
    };

    if response.media_type != "image" {
        bail!("Wrong media type");
    }

    let image_data = response.get_image_data(&agent)?;
    let ex = get_file_extension(&response.hdurl);

    let image_name = format!("{}.{}", response.date, ex);

    config.storage_dir.push(image_name);

    let mut image = File::create(config.storage_dir.clone())?;
    image.write_all(&image_data)?;
    fs::copy(config.storage_dir, config.background_path)?;
    println!("Saved!");

    Ok(())
}
