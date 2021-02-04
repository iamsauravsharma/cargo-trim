use std::{env, ffi::OsStr, fs, io::Read, path::Path};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{list_crate::CargoTomlLocation, utils::env_list};

// Stores config file information
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct ConfigFile {
    #[serde(default)]
    directory: Vec<String>,
    #[serde(default)]
    ignore_file_name: Vec<String>,
    #[serde(default)]
    scan_hidden_folder: bool,
    #[serde(default)]
    scan_target_folder: bool,
}

impl ConfigFile {
    // Perform initial config file actions
    pub(crate) fn init(app: &clap::ArgMatches, config_file: &Path) -> Self {
        let mut buffer = String::new();
        let mut file =
            fs::File::open(config_file.to_str().unwrap()).expect("failed to open config file");
        file.read_to_string(&mut buffer)
            .expect("failed to read config file");
        if buffer.is_empty() {
            let initial_config = Self::default();
            let serialize = toml::to_string_pretty(&initial_config)
                .expect("failed to convert ConfigFile to string");
            buffer.push_str(&serialize)
        }
        let mut deserialize_config: Self =
            toml::from_str(&buffer).expect("failed to convert string to ConfigFile");
        if app.is_present("config file modifier")
            || app.is_present("init")
            || app.is_present("clear")
            || app.is_present("remove")
        {
            // add working directory to config
            if app.is_present("init") {
                deserialize_config.add_directory(
                    &std::env::current_dir()
                        .expect("Current working directory is invalid")
                        .to_str()
                        .expect("failed to convert to str"),
                );
            }

            // Add new values to config file
            if let Some(values) = app.values_of("directory") {
                let path_separator = std::path::MAIN_SEPARATOR;
                for value in values {
                    let path = value.trim_end_matches(path_separator);
                    deserialize_config.add_directory(path);
                }
            }
            if let Some(values) = app.values_of("ignore_file_name") {
                let values = values.collect::<Vec<&str>>();
                for value in values {
                    deserialize_config.add_ignore_file_name(value)
                }
            }

            // clear working directory from config file
            if let Some(subcommand) = app.subcommand_matches("clear") {
                let dry_run = app.is_present("dry run") || subcommand.is_present("dry run");
                deserialize_config.remove_directory(
                    std::env::current_dir()
                        .expect("Current working directory is invalid")
                        .to_str()
                        .expect("Cannot convert to str"),
                    dry_run,
                );
            }

            // remove value from config file
            if let Some(subcommand) = app.subcommand_matches("remove") {
                let dry_run = app.is_present("dry run") || subcommand.is_present("dry run");
                if let Some(values) = subcommand.values_of("directory") {
                    for value in values {
                        let path_separator = std::path::MAIN_SEPARATOR;
                        let path = value.trim_end_matches(path_separator);
                        deserialize_config.remove_directory(path, dry_run);
                    }
                }
                if let Some(values) = subcommand.values_of("ignore_file_name") {
                    for value in values {
                        deserialize_config.remove_ignore_file_name(value, dry_run);
                    }
                }
            }

            // save struct in the config file
            let serialized = toml::to_string_pretty(&deserialize_config)
                .expect("ConfigFile cannot to converted to pretty toml");
            buffer.clear();
            buffer.push_str(&serialized);
            fs::write(config_file, buffer).expect("Failed to write a value to config file");
        }

        // analyze some env variable before setting value
        let env_directory = env_list("TRIM_DIRECTORY");
        for directory in env_directory {
            deserialize_config.add_ignore_file_name(&directory);
        }
        let env_ignore_file_name = env_list("TRIM_IGNORE_FILE_NAME");
        for ignore_file_name in env_ignore_file_name {
            deserialize_config.add_ignore_file_name(&ignore_file_name);
        }
        let env_scan_hidden_folder = env::var("TRIM_SCAN_HIDDEN_FOLDER");
        if let Ok(scan_hidden_folder) = env_scan_hidden_folder {
            if let Ok(new_val) = scan_hidden_folder.trim().parse::<bool>() {
                deserialize_config.set_scan_hidden_folder(new_val);
            }
        }
        let env_scan_target_folder = env::var("TRIM_SCAN_TARGET_FOLDER");
        if let Ok(scan_target_folder) = env_scan_target_folder {
            if let Ok(new_val) = scan_target_folder.trim().parse::<bool>() {
                deserialize_config.set_scan_target_folder(new_val);
            }
        }
        deserialize_config
    }

    // return vector of directory value in config file
    pub(crate) fn directory(&self) -> &Vec<String> {
        &self.directory
    }

    // return vector of ignore file name value in config file
    pub(crate) fn ignore_file_name(&self) -> &Vec<String> {
        &self.ignore_file_name
    }

    // scan hidden folder
    pub(crate) fn scan_hidden_folder(&self) -> bool {
        self.scan_hidden_folder
    }

    // scan target folder
    pub(crate) fn scan_target_folder(&self) -> bool {
        self.scan_target_folder
    }

    // set scan hidden folder value
    fn set_scan_hidden_folder(&mut self, value: bool) {
        self.scan_hidden_folder = value
    }

    // set scan target folder value
    fn set_scan_target_folder(&mut self, value: bool) {
        self.scan_target_folder = value
    }

    // add directory
    fn add_directory(&mut self, path: &str) {
        self.directory.push(path.to_string());
    }

    // add ignore file name
    fn add_ignore_file_name(&mut self, file_name: &str) {
        self.ignore_file_name.push(file_name.to_string());
    }

    // remove directory
    fn remove_directory(&mut self, path: &str, dry_run: bool) {
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".color("yellow"),
                "Removed".color("red"),
                path
            );
        } else {
            self.directory.retain(|data| data != path);
            println!("{} {:?}", "Removed".color("red"), path);
        }
    }

    // remove ignore file name
    fn remove_ignore_file_name(&mut self, file_name: &str, dry_run: bool) {
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".color("yellow"),
                "Removed".color("red"),
                file_name
            );
        } else {
            self.ignore_file_name.retain(|data| data != file_name);
            println!("{} {:?}", "Removed".color("red"), file_name);
        }
    }

    // List out cargo.toml file present directories by recursively analyze all
    // folder present in directory
    pub(crate) fn list_cargo_toml(&self, path: &Path) -> CargoTomlLocation {
        let mut cargo_trim_list = CargoTomlLocation::new();
        if path.exists() {
            if path.is_dir() {
                for entry in std::fs::read_dir(path)
                    .expect("failed to read directory while trying to find cargo.toml")
                {
                    let sub = entry.unwrap().path();
                    if sub.is_dir() {
                        if self.need_to_be_ignored(path) {
                            continue;
                        }
                        let kids_list = self.list_cargo_toml(&sub);
                        cargo_trim_list.append(kids_list);
                    }
                    if sub.is_file() && sub.file_name() == Some(OsStr::new("Cargo.toml")) {
                        cargo_trim_list.add_path(path.to_path_buf());
                    }
                }
            } else if path.is_file() && path.file_name() == Some(OsStr::new("Cargo.toml")) {
                cargo_trim_list.add_path(path.to_path_buf());
            }
        }
        cargo_trim_list
    }

    // check if directory should be scanned for listing crates or not
    fn need_to_be_ignored(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let is_file_name_ignored = self.ignore_file_name().contains(&file_name.to_owned());
        let file_is_hidden = file_name.starts_with('.') && !self.scan_hidden_folder();
        let target_dir_name = env::var("CARGO_BUILD_TARGET_DIR").unwrap_or_else(|_| {
            env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("target"))
        });
        let file_is_target = file_name == target_dir_name && !self.scan_target_folder();
        is_file_name_ignored || file_is_hidden || file_is_target
    }
}
