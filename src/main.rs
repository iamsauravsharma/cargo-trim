extern crate clap;
extern crate dirs;
extern crate find_folder;

use std::fs;

mod create_app;

fn main() {
    let config_dir = dirs::config_dir().unwrap();
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".cargo");
    home_dir.push("registry");
    let cargo_cache_config = format!(
        "{}/{}",
        config_dir.to_str().unwrap(),
        "cargo_cache_config.txt"
    );
    let _ = fs::File::open(&cargo_cache_config)
        .unwrap_or_else(|_| fs::File::create(&cargo_cache_config).unwrap());
    let app = create_app::app();
    if app.is_present("all") {
        fs::remove_dir_all(home_dir).unwrap();
    }
}
