extern crate rmp_serde as rmps;
use std::{collections::HashMap, env, fs, process::Command};

use colored::{control::set_override, Colorize};
use rmps::Serializer;
use serde::Serialize;

const LOC: &str = "https://raw.githubusercontent.com/Skytils/SkytilsMod-Data/main/constants/sellprices.json";
static LOGO: &str = r#"
██╗      ██████╗ ██╗    ██╗███████╗███████╗████████╗██████╗ ██╗███╗   ██╗███████╗
██║     ██╔═══██╗██║    ██║██╔════╝██╔════╝╚══██╔══╝██╔══██╗██║████╗  ██║██╔════╝
██║     ██║   ██║██║ █╗ ██║█████╗  ███████╗   ██║   ██████╔╝██║██╔██╗ ██║███████╗
██║     ██║   ██║██║███╗██║██╔══╝  ╚════██║   ██║   ██╔══██╗██║██║╚██╗██║╚════██║
███████╗╚██████╔╝╚███╔███╔╝███████╗███████║   ██║   ██████╔╝██║██║ ╚████║███████║
╚══════╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝╚══════╝   ╚═╝   ╚═════╝ ╚═╝╚═╝  ╚═══╝╚══════╝
"#;
fn main() {
    set_override(true);
    let res: String = LOGO
        .to_owned()
        .trim()
        .chars()
        .map(|x| {
            if x == '█' {
                "█".dimmed().green().to_string()
            } else if x == ' ' || x == '\n' {
                x.to_string()
            } else {
                x.to_string().bright_green().to_string()
            }
        })
        .collect::<_>();
    fs::write(format!("{}/logo.txt", &env::var("OUT_DIR").unwrap()), res).unwrap();
    let output = format!("{}/sellprices.json", &env::var("OUT_DIR").unwrap());
    let output_compressed = format!("{}/sellprices.bin", &env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed={output}");
    if fs::read(&output).is_err() {
        let _cmd = Command::new("curl")
            .arg("-o")
            .arg(&output)
            .arg(LOC)
            .output()
            .expect("failed to execute process");
    }
    if fs::read(&output).is_err() {
        fs::write(&output, r#"{}"#).unwrap();
    }
    if fs::read(&output_compressed).is_err() {
        let mut buf = Vec::new();
        serde_json::from_slice::<HashMap<String, f64>>(&fs::read(&output).unwrap())
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, v.round() as u64))
            .collect::<HashMap<String, u64>>()
            .serialize(&mut Serializer::new(&mut buf))
            .unwrap();
        fs::write(&output_compressed, buf).unwrap();
    }
}
