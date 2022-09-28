use colored::{control::set_override, Colorize};
use std::{env, fs, process::Command};
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
                "█".dimmed().green()
            } else {
                x.to_string().bright_green()
            }
        })
        .map(|x| format!("{}", x))
        .collect::<_>();
    fs::write(format!("{}/logo.txt", &env::var("OUT_DIR").unwrap()), res).unwrap();
    let output = format!("{}/sellprices.json", &env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed={}", output);
    let _cmd = Command::new("curl")
        .arg("-o")
        .arg(output)
        .arg(LOC)
        .output()
        .expect("failed to execute process");
}
