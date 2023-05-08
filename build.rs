use std::{collections::BTreeMap, env, fs, process::Command};

use colored::{control::set_override, Colorize};

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

    let file = fs::read(&output).unwrap();
    let file = serde_json::from_slice::<BTreeMap<String, f64>>(&file)
        .unwrap()
        .into_iter()
        .map(|(k, v)| {
            let v_u64 = v as u64;
            quote::quote! {
                (#k.to_owned(), #v_u64),
            }
        });

    let len = file.len();
    let default_prices = quote::quote! {
        pub fn get_prices_map() -> [(String, u64); #len] {
            [
                #(#file)*
            ]
        }
    };
    fs::write(
        format!("{}/prices_map.rs", &env::var("OUT_DIR").unwrap()),
        default_prices.to_string(),
    )
    .unwrap();
}
