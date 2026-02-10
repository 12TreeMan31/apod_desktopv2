# apod_desktopv2

A basic program that fetches the current image from [https://apod.nasa.gov].

## Install

Clone this repo and then run:
```bash
make install
```
Once installed you can then start the `apod.service`

## Config

XDG is respected, and a example config file can be found in the repo named `config`.
One thing to note when first running the program is that you must manually create the folders
that you want to write to. 

Also you can set `storage_dir` to null and images wont be saved.