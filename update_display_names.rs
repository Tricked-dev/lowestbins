extern crate rmp_serde as rmps;

use std::{collections::HashMap, fs};

use regex::Regex;
use rmps::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Item {
    internalname: String,
    displayname: String,
}

fn main() {
    if !std::path::Path::new("./neudata").exists() {
        std::process::Command::new("git")
            .args(&[
                "clone",
                "https://github.com/NotEnoughUpdates/NotEnoughUpdates-REPO",
                "neudata",
            ])
            .output()
            .unwrap();
    } else {
        std::process::Command::new("git")
            .args(&["pull"])
            .current_dir("./neudata")
            .output()
            .unwrap();
    }

    let re = Regex::new(r"§.").unwrap();
    let mut data = HashMap::new();
    for file in fs::read_dir("./neudata/items").unwrap() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() && file.path().extension().unwrap() == "json" {
            let file = fs::read(file.path()).unwrap();
            let file: Item = serde_json::from_slice(&file).unwrap();
            let displayname = re.replace_all(&file.displayname, "").to_string();
            data.insert(file.internalname, displayname);
        }
    }
    let mut buf = Vec::new();
    data.serialize(&mut Serializer::new(&mut buf)).unwrap();
    fs::write("./resources/display-names.bin", buf).unwrap();
}
