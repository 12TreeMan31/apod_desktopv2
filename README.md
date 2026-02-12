# apod_desktopv2

A basic program that fetches the current image from https://apod.nasa.gov.

## Install

Clone this repo and then run:
```bash
make install
```
Once installed you can then start `apod.service`

## Usage

### Arguments

Currently there are only two different arguments:
- `--config`: Specifies where the config file is located
- `--save`: Creates a simlink in `favorite_dir` of the newest image downloaded

### Config

XDG is respected, and a example config file can be found in the repo named `config`.
One thing to note when first running the program is that you must manually create the folders that you want to write to. 

## API Usage

If you plan on using this program I suggest creating your own [api key](https://api.nasa.gov/).

When running this program, there will be at most 2 API calls preformed. The first one is getting todays image info, and the second one is fetching the image itself.