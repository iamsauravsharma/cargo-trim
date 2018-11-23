use std::{fs, path::PathBuf};

pub(super) fn create_dir() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let mut config_dir = dirs::config_dir().unwrap();
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".cargo");
    home_dir.push("registry");
    let registry_dir = home_dir;
    config_dir.push("cargo_cache_config.json");

    // If config file does not exists create one config file
    if !config_dir.exists() {
        fs::File::create(config_dir.to_str().unwrap()).unwrap();
    }

    let mut cache_dir = registry_dir.clone();
    cache_dir.push("cache");
    let mut src_dir = registry_dir.clone();
    src_dir.push("src");
    let mut index_dir = registry_dir.clone();
    index_dir.push("index");

    (config_dir, registry_dir, cache_dir, index_dir, src_dir)
}
