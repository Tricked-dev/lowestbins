# Lowestbins-rs

A fast and effecient lowestbins implementation this uses parrallel requests to fetch all lowestbins in less than **4 SECONDS** and uses the hyper server allowing for practically unlimited requests per second while only using 70mb of ram!

## Building

- If you're on Linux, you can go to releases and download the binary
- Windows/MacOS: you need to install cargo and run cargo build --release and the exe/binary should be in the target/release/ folder.

## Features

- NBT parsing
- Fetching auctions and returning the lowest bin
- hyper server
- \+prob more things

> Licensed under the [GNU General Public License v2.0](./LICENSE) License
