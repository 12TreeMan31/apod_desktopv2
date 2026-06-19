# Space Image Downloader (sid-bg)

A basic program that fetches the current image from https://apod.nasa.gov.

## Install

Clone this repo and then run:
```bash
make install
```
Once installed you can then start `apod.service`

## Usage

Sid-bg is meant to be used with other tools like `swaybg` or `awww` in order to set your background.

Example of setting the background to todays image
```bash
BG=`sid-bg` && swaybg $BG
```

### Arguments

Currently there are only three different arguments:
- `--config`: Specifies where the config file is located
- `--path`: Print the path of the current background image
- `--random`: Sets a random downloaded image as the current background

### Config

XDG is respected, and a example config file is provided. The parsing code can also be found in `src/config.rs` if you need more detail.

## API Usage

I suggest creating your own [api key](https://api.nasa.gov/).

When running this program, there will be at most 2 API calls preformed. The first one is getting todays image info, and the second one is fetching the image itself.