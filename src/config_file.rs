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

    // return vector of exclude value in config file
    pub(crate) fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }

    // return vector of include value in config file
    pub(crate) fn include(&self) -> &Vec<String> {
        &self.include
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
}

// Function to modify config file or read config file
pub(crate) fn config_file(app: &clap::ArgMatches, config_dir: &PathBuf) -> ConfigFile {
    let mut buffer = String::new();
    let mut file =
        fs::File::open(config_dir.to_str().unwrap()).expect("failed to open config dir folder");
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

        // Add new value in config file
        if let Some(value) = app.value_of("set directory") {
            let path_separator = std::path::MAIN_SEPARATOR;
            let path = value.trim_end_matches(path_separator);
            deserialize_config.mut_directory().push(path.to_string());
        }
        if let Some(value) = app.value_of("exclude") {
            deserialize_config.mut_exclude().push(value.to_string());
        }
        if let Some(value) = app.value_of("include") {
            deserialize_config.mut_include().push(value.to_string());
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
            if let Some(value) = subcommand.value_of("directory") {
                let path_separator = std::path::MAIN_SEPARATOR;
                let path = value.trim_end_matches(path_separator);
                remove_item_crate(deserialize_config.mut_directory(), path, dry_run);
            }
            if let Some(value) = subcommand.value_of("exclude") {
                remove_item_crate(deserialize_config.mut_exclude(), value, dry_run);
            }
            if let Some(value) = subcommand.value_of("include") {
                remove_item_crate(deserialize_config.mut_include(), value, dry_run);
            }
        }

        let serialized = serde_json::to_string_pretty(&deserialize_config)
            .expect("ConfigFile cannot to converted to pretty json");
        buffer.clear();
        buffer.push_str(&serialized);
        fs::write(config_dir, buffer).expect("Failed to write a value to config file");
    }
    deserialize_config
}

// helper function to help in removing certain value from a config file
fn remove_item_crate(data: &mut Vec<String>, value: &str, dry_run: bool) {
    if dry_run {
        println!("{} {} {:?}", "Dry run:".color("yellow"), "removed".color("red"), value);
    } else {
        data.retain(|data| data != value);
        println!("{} {:?}", "Removed".color("red"), value);
    }
}
