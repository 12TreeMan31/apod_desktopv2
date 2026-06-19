use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use serde::Deserialize;
use std::ffi::OsString;
use std::io;
use ureq::Agent;

const APOD_URL: &str = "https://api.nasa.gov/planetary/apod";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Response {
    pub copyright: Option<String>,
    pub date: String,
    pub explanation: String,
    pub hdurl: String,
    pub url: String,
    pub media_type: String,
    pub title: String,
}

impl Response {
    pub fn make_request(agent: &Agent, api_key: &str) -> Result<Self, ureq::Error> {
        let mut response = agent.get(APOD_URL).query("api_key", api_key).call()?;

        let json: Response = response.body_mut().read_json()?;
        Ok(json)
    }

    pub fn get_image_bytes(&self, agent: &Agent) -> Result<Vec<u8>, ureq::Error> {
        let mut resp = agent.get(&self.hdurl).call()?;
        let data = resp.body_mut().read_to_vec()?;

        Ok(data)
    }

    pub fn image_name(&self) -> OsString {
        format!("{}.jpg", self.date).into()
    }

    pub fn set_image_metadata(&self, image_data: &mut Vec<u8>) -> io::Result<()> {
        let mut metadata = Metadata::new();
        metadata.set_tag(ExifTag::ImageDescription(self.explanation.clone()));
        metadata.set_tag(ExifTag::CreateDate(self.date.clone()));

        if let Some(copyright) = self.copyright.as_ref() {
            metadata.set_tag(ExifTag::Copyright(copyright.clone()));
        }

        metadata.write_to_vec(image_data, FileExtension::JPEG)?;
        Ok(())
    }
}
