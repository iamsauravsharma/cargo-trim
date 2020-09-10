use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::PathBuf};

// Stores config file information
#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    directory: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    include: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    exclude: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ignore_file_name: Vec<String>,
}

impl ConfigFile {
    // Create new config file
    pub(crate) fn new() -> Self {
        Self {
            directory: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            ignore_file_name: Vec::new(),
        }
    }

    // return vector of directory value in config file
    pub(crate) fn directory(&self) -> &Vec<String> {
        &self.directory
    }

    // return vector of exclude value in config file
    pub(crate) fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }

    // return vector of include value in config file
    pub(crate) fn include(&self) -> &Vec<String> {
        &self.include
    }

    // return vector of ignore file name value in config file
    pub(crate) fn ignore_file_name(&self) -> &Vec<String> {
        &self.ignore_file_name
    }

    // return mutable reference of directory value
    pub(crate) fn mut_directory(&mut self) -> &mut Vec<String> {
        &mut self.directory
    }

    // return mutable reference of exclude value
    pub(crate) fn mut_exclude(&mut self) -> &mut Vec<String> {
        &mut self.exclude
    }

    // return mutable reference of include value
    pub(crate) fn mut_include(&mut self) -> &mut Vec<String> {
        &mut self.include
    }

    // return mutable reference of ignore file name value
    pub(crate) fn mut_ignore_file_name(&mut self) -> &mut Vec<String> {
        &mut self.ignore_file_name
    }
}

// Function to modify config file or read config file
#[allow(clippy::too_many_lines)]
pub(crate) fn config_file(app: &clap::ArgMatches, config_file: &PathBuf) -> ConfigFile {
    let mut buffer = String::new();
    let mut file =
        fs::File::open(config_file.to_str().unwrap()).expect("failed to open config dir folder");
    file.read_to_string(&mut buffer)
        .expect("failed to read config file string");
    if buffer.is_empty() {
        let initial_config = ConfigFile::new();
        let serialize = toml::to_string_pretty(&initial_config)
            .expect("failed to convert ConfigFile to string");
        buffer.push_str(&serialize)
    }
    let mut deserialize_config: ConfigFile =
        toml::from_str(&buffer).expect("failed to convert string to ConfigFile");
    if app.is_present("config file modifier")
        || app.is_present("init")
        || app.is_present("clear")
        || app.is_present("remove")
    {
        // add working directory to config
        if app.is_present("init") {
            deserialize_config.mut_directory().push(
                std::env::current_dir()
                    .expect("Current working directory is invalid")
                    .to_str()
                    .expect("failed to convert current directory Path to str")
                    .to_string(),
            );
        }

        // Add new values to config file
        if let Some(values) = app.values_of("directory") {
            let path_separator = std::path::MAIN_SEPARATOR;
            for value in values {
                let path = value.trim_end_matches(path_separator);
                deserialize_config.mut_directory().push(path.to_string());
            }
        }
        if let Some(values) = app.values_of("exclude") {
            let values: Vec<String> = values
                .collect::<Vec<&str>>()
                .iter()
                .map(|&s| s.to_string())
                .collect();
            deserialize_config.mut_exclude().extend(values);
        }
        if let Some(values) = app.values_of("include") {
            let values: Vec<String> = values
                .collect::<Vec<&str>>()
                .iter()
                .map(|&s| s.to_string())
                .collect();
            deserialize_config.mut_include().extend(values);
        }
        if let Some(values) = app.values_of("ignore_file_name") {
            let values: Vec<String> = values
                .collect::<Vec<&str>>()
                .iter()
                .map(|&s| s.to_string())
                .collect();
            deserialize_config.mut_ignore_file_name().extend(values);
        }

        // clear working directory from config file
        if let Some(subcommand) = app.subcommand_matches("clear") {
            let dry_run = app.is_present("dry run") || subcommand.is_present("dry run");
            remove_item_crate(
                deserialize_config.mut_directory(),
                std::env::current_dir()
                    .expect("Current working directory is invalid")
                    .to_str()
                    .expect("failed to convert current directory Path to str"),
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
                    remove_item_crate(deserialize_config.mut_directory(), path, dry_run);
                }
            }
            if let Some(values) = subcommand.values_of("exclude") {
                for value in values {
                    remove_item_crate(deserialize_config.mut_exclude(), value, dry_run);
                }
            }
            if let Some(values) = subcommand.values_of("include") {
                for value in values {
                    remove_item_crate(deserialize_config.mut_include(), value, dry_run);
                }
            }
            if let Some(values) = subcommand.values_of("ignore_file_name") {
                for value in values {
                    remove_item_crate(deserialize_config.mut_ignore_file_name(), value, dry_run);
                }
            }
        }

        // save struct in the config file
        let serialized = toml::to_string_pretty(&deserialize_config)
            .expect("ConfigFile cannot to converted to pretty json");
        buffer.clear();
        buffer.push_str(&serialized);
        fs::write(config_file, buffer).expect("Failed to write a value to config file");
    }
    deserialize_config
}

// helper function to help in removing certain value from a config file
fn remove_item_crate(data: &mut Vec<String>, value: &str, dry_run: bool) {
    if dry_run {
        println!(
            "{} {} {:?}",
            "Dry run:".color("yellow"),
            "Removed".color("red"),
            value
        );
    } else {
        data.retain(|data| data != value);
        println!("{} {:?}", "Removed".color("red"), value);
    }
}
