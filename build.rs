use std::{env, process::Command};

const LOC: &str = "https://raw.githubusercontent.com/Skytils/SkytilsMod-Data/main/constants/sellprices.json";
fn main() {
    let output = format!("{}/sellprices.json", &env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed={}", output);
    let _cmd = Command::new("curl")
        .arg("-o")
        .arg(output)
        .arg(LOC)
        .output()
        .expect("failed to execute process");
}
