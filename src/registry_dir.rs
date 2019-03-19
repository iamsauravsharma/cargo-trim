use crate::{ConfigFile, CrateDetail};
use clap::ArgMatches;
use colored::*;
use std::{fs, path::Path};

// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir {
    cache_dir: String,
    src_dir: String,
}

impl RegistryDir {
    // Create new RegistryDir
    pub(crate) fn new(cache_dir: &Path, src_dir: &Path) -> Self {
        let cache_dir = open_github_folder(cache_dir);
        let src_dir = open_github_folder(src_dir);
        Self { cache_dir, src_dir }
    }

    // Remove crate from src & cache directory
    pub(crate) fn remove_crate(&self, crate_name: &str) {
        remove_crate(Path::new(&self.cache_dir), crate_name);
        remove_crate(Path::new(&self.src_dir), crate_name);
        println!("{} {:?}", "Removed".red(), crate_name);
    }

    // Get out src_dir path
    pub(crate) fn src(&self) -> &String {
        &self.src_dir
    }

    // Get out src_dir path
    pub(crate) fn cache(&self) -> &String {
        &self.cache_dir
    }

    // Remove list of crates
    pub(crate) fn remove_crate_list(&self, crate_detail: &CrateDetail, list: &[String]) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find(crate_name, "REGISTRY")
        }
        size_cleaned
    }

    // Remove all crates from registry folder
    pub(crate) fn remove_all(
        &self,
        config_file: &ConfigFile,
        app: &ArgMatches,
        crate_name: &str,
        crate_detail: &CrateDetail,
    ) -> f64 {
        let mut cmd_include = Vec::new();
        let mut cmd_exclude = Vec::new();
        let crate_name = &crate_name.to_string();
        let mut sized_cleaned = 0.0;

        // Provide one time include crate list for other flag
        if app.is_present("include") {
            let value = app.value_of("include").unwrap().to_string();
            cmd_include.push(value);
        }

        // Provide one time exclude crate list for other flag
        if app.is_present("exclude") {
            let value = app.value_of("include").unwrap().to_string();
            cmd_exclude.push(value);
        }

        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        if cmd_include.contains(crate_name) || read_include.contains(crate_name) {
            self.remove_crate(crate_name);
            sized_cleaned += crate_detail.find_size_registry_all(crate_name);
        }
        if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
            self.remove_crate(crate_name);
            sized_cleaned += crate_detail.find_size_registry_all(crate_name);
        }
        sized_cleaned
    }
}

// Use to open github folder present inside src and cache folder
fn open_github_folder(path: &Path) -> String {
    let mut path_buf = path.to_path_buf();
    path_buf.push("github.com-1ecc6299db9ec823");
    path_buf.to_str().unwrap().to_string()
}

// Remove crates which name is provided to delete
fn remove_crate(path: &Path, value: &str) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.to_str().unwrap().contains(value) {
            if path.is_file() {
                fs::remove_file(&path).unwrap();
            } else if path.is_dir() {
                fs::remove_dir_all(&path).unwrap();
            }
        }
    }
}
