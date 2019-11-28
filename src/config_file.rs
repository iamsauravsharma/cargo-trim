use colored::*;
use serde::{Deserialize, Serialize};
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
    file.read_to_string(&mut buffer)
        .expect("failed to read config file string");
    if buffer.is_empty() {
        let initial_config = ConfigFile::new();
        let serialize =
            serde_json::to_string(&initial_config).expect("failed to convert ConfigFile to string");
        buffer.push_str(&serialize)
    }
    let mut deserialize_config: ConfigFile =
        serde_json::from_str(&buffer).expect("failed to convert string to ConfigFile");

    // add working directory to config
    if app.is_present("init") {
        deserialize_config.directory.push(
            std::env::current_dir()
                .expect("Current working directory is invalid")
                .to_str()
                .expect("failed to convert current directory Path to str")
                .to_string(),
        );
    }

    // Add new value in config file
    for &name in &["set directory", "exclude", "include"] {
        if app.is_present(name) {
            let value = app
                .value_of(name)
                .expect("No value is present for remove value from config file flag");
            if name == "set directory" {
                deserialize_config.directory.push(value.to_string());
            }
            if name == "exclude" {
                deserialize_config.exclude.push(value.to_string());
            }
            if name == "include" {
                deserialize_config.include.push(value.to_string());
            }
        }
    }

    // clear working directory from config file
    if app.is_present("clear") {
        let subcommand = app.subcommand_matches("clear").unwrap();
        let dry_run = app.is_present("dry-run") || subcommand.is_present("dry run");
        remove_item_crate(
            &mut deserialize_config.directory,
            std::env::current_dir()
                .expect("Current working directory is invalid")
                .to_str()
                .expect("failed to convert current directory Path to str"),
            dry_run,
        );
    }

    // remove value from config file
    if app.is_present("remove") {
        let subcommand = app.subcommand_matches("remove").unwrap();
        for &name in &["directory", "exclude", "include"] {
            let dry_run = app.is_present("dry run") || subcommand.is_present("dry run");
            if subcommand.is_present(name) {
                let value = subcommand
                    .value_of(name)
                    .expect("No value is present for remove value from config file flag")
                    .to_string();
                if name == "directory" {
                    remove_item_crate(&mut deserialize_config.directory, &value, dry_run);
                }
                if name == "exclude" {
                    remove_item_crate(&mut deserialize_config.exclude, &value, dry_run);
                }
                if name == "include" {
                    remove_item_crate(&mut deserialize_config.include, &value, dry_run);
                }
            }
        }
    }

    let serialized = serde_json::to_string_pretty(&deserialize_config)
        .expect("ConfigFile cannot to converted to pretty json");
    buffer.clear();
    buffer.push_str(&serialized);
    fs::write(config_dir, buffer).expect("Failed to write a value to config file");
    deserialize_config
}

// helper function to help in removing certain value from a config file
fn remove_item_crate(data: &mut Vec<String>, value: &str, dry_run: bool) {
    if dry_run {
        println!("{} {} {:?}", "Dry run:".yellow(), "removed".red(), value);
    } else {
        data.retain(|data| data != value);
        println!("{} {:?}", "Removed".red(), value);
    }
}
