#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use jiu::Config;
use std::fs;

fn main() {
    let toml = fs::read_to_string(".jiu.toml").unwrap(); // Prototyping
    let config: Config = match toml::de::from_str(&toml) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing .jiu.toml: {}", e);
            std::process::exit(1);
        }
    };
    println!("{config:#?}");
}
