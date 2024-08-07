![](assets/logo.png)

<div align="center">

<img alt="Logo" src=".github/assets/logo.png" width="200">

[![GitHub issues](https://img.shields.io/github/issues/Tricked-dev/lowestbins)](https://github.com/Tricked-dev/lowestbins/issues) [![GitHub forks](https://img.shields.io/github/forks/Tricked-dev/lowestbins)](https://github.com/Tricked-dev/lowestbins/network)
[![GitHub stars](https://img.shields.io/github/stars/Tricked-dev/lowestbins)](https://github.com/Tricked-dev/lowestbins/stargazers)
[![GitHub license](https://img.shields.io/github/license/Tricked-dev/lowestbins)](https://github.com/Tricked-dev/lowestbins/blob/master/LICENSE)
![Website](https://img.shields.io/website?url=https%3A%2F%2Flb.tricked.dev%2F)
[![Discord](https://img.shields.io/discord/748956745409232945)](https://discord.gg/mY8zTARu4g)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/tricked-dev)](https://github.com/sponsors/Tricked-dev)

[![forthebadge](https://forthebadge.com/images/badges/ctrl-c-ctrl-v.svg)](https://forthebadge.com)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

# Lowestbins-rs

</div>

A fast and effecient lowestbins implementation this uses parrallel requests to fetch all lowestbins in less than **2.5 SECONDS** and uses the hyper server allowing for practically unlimited requests per second while only using 50mb of ram!  
Lowestbins-rs is used by [Skytils](https://github.com/Skytils/SkytilsMod) for the lowestbins feature.

## Hosted Instance

- [lb.tricked.dev/](https://lb.tricked.dev/)

If you are using this in a project please credit me! or sponsor me on github [github.com/sponsors/Tricked-dev](https://github.com/sponsors/Tricked-dev)

## Using in your code

### Javascript

**node:**

```js
const fetch = require("undici"); // node-fetch also works
let json = await fetch("https://lb.tricked.dev/lowestbins").then((res) =>
  res.json()
);

console.log(json["ENCHANTMENT_ULTIMATE_SWARM_2"]);
```

**web/deno:**

```js
let json = await fetch("https://lb.tricked.dev/lowestbins").then((res) =>
  res.json()
);
console.log(json["ENCHANTMENT_ULTIMATE_SWARM_2"]);
```

### Python

```py
import requests
json = requests.get("https://lb.tricked.dev/lowestbins").json()
print(json["ENCHANTMENT_ULTIMATE_SWARM_2"])
```

### Api endpoints

#### `/lowestbins` | `/lowestbins.json` | `/auctions/lowestbins`

> Returns a json object with all lowestbins

#### `/lowestbins.txt`

> Returns a text file with all lowestbins easier to parse and smaller in size

#### `/auction/{item}` | `lowestbin/{item}`

> Returns the price of a item as number this endpoint can be used to get more up to date prices due to it not being cached as often

#### `/metrics`

> Returns prometheus metrics of the prices

#### `/`

> Returns basic data about the api and update time

## Usage

### Docker

```bash
docker run  --name lowestbins -p 8080:8080 -e HOST=0.0.0.0 -e UPDATE_SECONDS=120 -d ghcr.io/tricked-dev/lowestbins:latest
```

## Building

- If you're on Linux, you can go to releases and download the binary
- Windows/MacOS: you need to install cargo and run `cargo build --release` and the exe/binary should be in the `./target/release/lowestbins`.

## Config Options

Env variables

```env
PORT # The port to run the server on
HOST # The host to run the server on
SAVE_TO_DISK # set to 0 to not save the auctions to disk
OVERWRITES # Overwrite values format: `BLESSED_BAIT:200,ROCK_CANDY:6000,NON_EXISTENT_ITEM:200`
UPDATE_SECONDS # The amount of seconds to wait before updating the lowestbins
WEBHOOK_URL # The webhook url used for reporting the requests (discord/discord compatible)
RUST_LOG # The log level lowestbins=debug recommended
```

## Features

- NBT parsing
- Fetching auctions and returning the lowest bin
- hyper server
- skytils compatible
- metrics endpoint `/metrics`
- rust

## Todo

- use quiche for faster quests
- use cargo pgo to optimize binaries

> Licensed under the [Apache 2](./LICENSE) License
