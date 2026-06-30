use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use miniserde::{Deserialize, json};
use std::borrow::Cow;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use ureq::Agent;

const APOD_URL: &str = "https://api.nasa.gov/planetary/apod";

// I might have make this a little overkill, but I think its cool
#[derive(Debug)]
pub struct Query<'a>(Vec<(&'static str, Cow<'a, str>)>);

impl<'a> Query<'a> {
    pub fn fetch_date(api_key: &'a str, date: Option<&'a str>) -> Self {
        let mut query = vec![];
        if let Some(date) = date {
            query.push(("date", Cow::Borrowed(date)));
        }

        query.push(("api_key", Cow::Borrowed(api_key)));

        Query(query)
    }

    pub fn fetch_range(api_key: &'a str, start: &'a str, end: &'a str) -> Self {
        let query = vec![
            ("api_key", Cow::Borrowed(api_key)),
            ("start_date", Cow::Borrowed(start)),
            ("end_date", Cow::Borrowed(end)),
        ];

        Query(query)
    }

    pub fn fetch_random(api_key: &'a str, n: u32) -> Self {
        let query = vec![
            ("api_key", Cow::Borrowed(api_key)),
            ("count", Cow::Owned(n.to_string())),
        ];

        Query(query)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Response {
    copyright: Option<String>,
    date: String,
    explanation: String,
    hdurl: Option<String>,
    url: Option<String>,
    media_type: String,
    title: String,
    service_version: String,
}

impl Response {
    pub fn make_request(agent: Agent, query: Query) -> Result<Vec<Self>, ureq::Error> {
        let raw = agent
            .get(APOD_URL)
            .query_pairs(query.0.into_iter())
            .call()?;

        let mut body = raw.into_body().read_to_string()?;

        // I would like to use this function for ANY type of request the API can throw out.
        // That means we must make all json an array since thats what random and range return.
        if !body.trim_start().starts_with('[') {
            body = format!("[{}]", body);
        }

        let res: Vec<Response> = json::from_str(&body).expect("Could not parse json");
        Ok(res)
    }

    /// Downloads image in current request into the directory. It is important to verify
    /// `media type == "image"` before calling this function.
    pub fn download_image(&self, agent: Agent, dir: &Path) -> io::Result<PathBuf> {
        let url = self
            .hdurl
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "hdurl is not set!"))?;

        let mut res = agent.get(url).call().map_err(|e| {
            io::Error::new(
                io::ErrorKind::ConnectionRefused,
                format!("Unable to fetch image data: {e}"),
            )
        })?;
        let mut reader = res.body_mut().as_reader();

        let path = dir.join(self.image_name());
        let mut file = File::create(&path)?;

        io::copy(&mut reader, &mut file)?;

        Ok(path)
    }

    /// Writes apod image metadata to image at `image_path`
    pub fn write_metadata(&self, image_path: &Path) -> io::Result<()> {
        let mut metadata = Metadata::new();
        metadata.set_tag(ExifTag::ImageDescription(self.explanation.clone()));
        metadata.set_tag(ExifTag::CreateDate(self.date.clone()));

        if let Some(copyright) = self.copyright.as_ref() {
            metadata.set_tag(ExifTag::Copyright(copyright.clone()));
        }

        metadata.write_to_file(image_path)?;
        Ok(())
    }

    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    pub fn image_name(&self) -> OsString {
        format!("{}.jpg", self.date).into()
    }
}
