extern crate clap;
extern crate dirs;

use std::fs::File;

mod create_app;

fn main() {
    let config_dir = dirs::config_dir().unwrap();
    let home_dir = dirs::home_dir().unwrap();
    let cargo_cache_config = format!(
        "{}/{}",
        config_dir.to_str().unwrap(),
        "cargo_cache_config.txt"
    );
    let _registry_folder = format!("{}/.cargo/registry", home_dir.to_str().unwrap());
    let _ = File::open(&cargo_cache_config)
        .unwrap_or_else(|_| File::create(&cargo_cache_config).unwrap());
    create_app::app();
}
