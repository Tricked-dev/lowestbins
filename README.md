# Lowestbins-rs

A fast and effecient lowestbins implementation this uses parrallel requests to fetch all lowestbins in less than **3.5 SECONDS** and uses the hyper server allowing for practically unlimited requests per second while only using 50mb of ram!

## Hosted Instance

- [lb.tricked.pro/](https://lb.tricked.pro/)

If you are using this in a project please credit me! or sponsor me on github [github.com/sponsors/Tricked-dev](https://github.com/sponsors/Tricked-dev)

## Building

- If you're on Linux, you can go to releases and download the binary
- Windows/MacOS: you need to install cargo and run cargo build --release and the exe/binary should be in the target/release/ folder.

## Config Options

Env variables

```env
PORT # The port to run the server on
HOST # The host to run the server on
SAVE_TO_DISK # set to 0 to not save the auctions to disk
PARRALELL # The amount of parrallel requests to make (advanced usage)
OVERWRITES # Overwrite values format: `BLESSED_BAIT:200,ROCK_CANDY:6000,NON_EXISTENT_ITEM:200`
UPDATE_SECONDS # The amount of seconds to wait before updating the lowestbins
WEBHOOK_URL # The webhook url used for reporting the requests (discord/discord compatible)
```

## Features

- NBT parsing
- Fetching auctions and returning the lowest bin
- hyper server
- \+prob more things

> Licensed under the [Apache 2](./LICENSE) License

```

```
