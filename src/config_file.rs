use std::{
    env,
    ffi::OsStr,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::list_crate::CargoTomlLocation;

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
    #[serde(skip)]
    config_file: PathBuf,
}

impl ConfigFile {
    // Perform initial config file actions
    pub(crate) fn init(config_file: &Path) -> Result<Self> {
        let mut buffer = String::new();
        let mut file = fs::File::open(config_file).context("failed to open config file")?;
        file.read_to_string(&mut buffer)
            .context("failed to read config file")?;
        if buffer.is_empty() {
            let initial_config = Self::default();
            let serialize = toml::to_string_pretty(&initial_config)
                .context("failed to convert Config to string")?;
            buffer.push_str(&serialize)
        }
        let mut deserialize_config: Self =
            toml::from_str(&buffer).context("failed to convert string to Config")?;
        deserialize_config.config_file = config_file.to_path_buf();
        Ok(deserialize_config)
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

    // add directory
    pub(crate) fn add_directory(&mut self, path: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            println!("{} Added {:?}", "Dry run:".color("yellow"), path);
        } else {
            self.directory.push(path.to_string());
            self.save_to_config_file()?;
            println!("{} {:?}", "Added".color("red"), path);
        }
        Ok(())
    }

    // add ignore file name
    pub(crate) fn add_ignore_file_name(&mut self, file_name: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            println!("{} Added {:?}", "Dry run:".color("yellow"), file_name);
        } else {
            self.ignore_file_name.push(file_name.to_string());
            self.save_to_config_file()?;
            println!("{} {:?}", "Added".color("red"), file_name);
        }
        Ok(())
    }

    // remove directory
    pub(crate) fn remove_directory(&mut self, path: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".color("yellow"),
                "Removed".color("red"),
                path
            );
        } else {
            self.directory.retain(|data| data != path);
            self.save_to_config_file()?;
            println!("{} {:?}", "Removed".color("red"), path);
        }
        Ok(())
    }

    // remove ignore file name
    pub(crate) fn remove_ignore_file_name(&mut self, file_name: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".color("yellow"),
                "Removed".color("red"),
                file_name
            );
        } else {
            self.ignore_file_name.retain(|data| data != file_name);
            self.save_to_config_file()?;
            println!("{} {:?}", "Removed".color("red"), file_name);
        }
        Ok(())
    }

    // List out cargo.toml file present directories by recursively analyze all
    // folder present in directory
    pub(crate) fn list_cargo_toml(&self, path: &Path) -> Result<CargoTomlLocation> {
        let mut cargo_trim_list = CargoTomlLocation::new();
        if path.exists() {
            if path.is_dir() {
                for entry in std::fs::read_dir(path)
                    .context("failed to read directory while trying to find cargo.toml")?
                {
                    let sub = entry?.path();
                    if sub.is_dir() {
                        if self.need_to_be_ignored(path) {
                            continue;
                        }
                        let kids_list = self.list_cargo_toml(&sub)?;
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
        Ok(cargo_trim_list)
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

    // save struct in the config file
    fn save_to_config_file(&self) -> Result<()> {
        let mut buffer = String::new();
        let serialized =
            toml::to_string_pretty(&self).context("Config cannot to converted to pretty toml")?;
        buffer.push_str(&serialized);
        fs::write(&self.config_file, buffer).context("Failed to write a value to config file")?;
        Ok(())
    }
}
