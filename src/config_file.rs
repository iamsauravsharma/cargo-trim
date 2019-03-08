use serde_derive::{Deserialize, Serialize};
use std::{fs, io::Read, path::PathBuf};

// Stores config file information
#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigFile {
    directory: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
}

impl ConfigFile {
    // Create new config file
    pub(crate) fn new() -> Self {
        Self {
            directory: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    // return vector of directory value in config file
    pub(crate) fn directory(&self) -> &Vec<String> {
        &self.directory
    }

    // return vector of include value in config file
    pub(crate) fn include(&self) -> &Vec<String> {
        &self.include
    }

    // return vector of exclude value in config file
    pub(crate) fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

// Function to modify config file or read config file
pub(crate) fn modify_config_file(
    file: &mut fs::File,
    app: &clap::ArgMatches,
    config_dir: &PathBuf,
) -> ConfigFile {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    if buffer.is_empty() {
        let initial_config = ConfigFile::new();
        let serialize = serde_json::to_string(&initial_config).unwrap();
        buffer.push_str(&serialize)
    }
    let mut deserialized: ConfigFile = serde_json::from_str(&buffer).unwrap();

    // add working directory to config
    if app.is_present("init") {
        deserialized.directory.push(
            std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        )
    }
    // Add new value in config file
    for &name in &["set directory", "exclude-conf", "include-conf"] {
        if app.is_present(name) {
            let value = app.value_of(name).unwrap();
            if name == "set directory" {
                deserialized.directory.push(value.to_string());
            }
            if name == "exclude-conf" {
                deserialized.exclude.push(value.to_string());
            }
            if name == "include-conf" {
                deserialized.include.push(value.to_string());
            }
        }
    }

    // clear working directory from config file
    if app.is_present("clear") {
        remove_item_crate(
            &mut deserialized.directory,
            std::env::current_dir().unwrap().to_str().unwrap(),
        )
    }

    // remove value from config file
    if app.is_present("remove") {
        let subcommand = app.subcommand_matches("remove").unwrap();
        for &name in &["directory", "exclude", "include"] {
            if subcommand.is_present(name) {
                let value = subcommand.value_of(name).unwrap().to_string();
                if name == "directory" {
                    remove_item_crate(&mut deserialized.directory, &value);
                }
                if name == "exclude" {
                    remove_item_crate(&mut deserialized.exclude, &value);
                }
                if name == "include" {
                    remove_item_crate(&mut deserialized.include, &value);
                }
            }
        }
    }

    let serialized = serde_json::to_string_pretty(&deserialized).unwrap();
    buffer.clear();
    buffer.push_str(&serialized);
    fs::write(config_dir, buffer).unwrap();
    deserialized
}

// helper function to help in removing certain value from a config file
fn remove_item_crate(data: &mut Vec<String>, value: &str) {
    data.retain(|data| data != value);
}
