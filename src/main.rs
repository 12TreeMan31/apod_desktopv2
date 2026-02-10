// SPDX-License-Identifier: GPL-2.0

use anyhow::Result;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Config {
    api_key: String,
    storage_dir: Option<PathBuf>,
    background_path: PathBuf,
}

impl Config {
    fn load(path: String) -> Result<Self> {
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
    fn make_request(api_key: &str) -> Result<Self> {
        let response: Response = ureq::get("https://api.nasa.gov/planetary/apod")
            .query("api_key", api_key)
            .call()?
            .body_mut()
            .read_json()?;

        Ok(response)
    }
}

fn main() {
    let Ok(config) = Config::load("config".to_string()) else {
        panic!("Unable to read config! Please make sure it exsit and is in valid json.");
    };
    println!("Read config");
    let Ok(response) = Response::make_request(&config.api_key) else {
        panic!("Unable to get get todays image! Is the api key valid?");
    };
    println!("Got response");
    let image_data = ureq::get(response.hdurl)
        .call()
        .unwrap()
        .body_mut()
        .read_to_vec()
        .unwrap();
    println!("Got image data");
    let image_name = format!("{}.jpg", response.date);

    if let Some(mut storage_dir) = config.storage_dir {
        storage_dir.push(image_name);

        let mut image = File::create(storage_dir.clone()).unwrap();
        image.write_all(&image_data).unwrap();
        fs::copy(storage_dir, config.background_path).unwrap();
    }
}
