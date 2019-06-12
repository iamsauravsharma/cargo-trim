use crate::{ConfigFile, CrateDetail};
use clap::ArgMatches;
#[cfg(feature = "colored-output")]
use colored::*;
use std::{fs, path::Path};

// Store git dir folder information
pub(crate) struct GitDir {
    checkout_dir: String,
    db_dir: String,
}

impl GitDir {
    // createnew GitDir
    pub(crate) fn new(checkout_dir: &Path, db_dir: &Path) -> Self {
        let checkout_dir = checkout_dir.to_str().unwrap().to_string();
        let db_dir = db_dir.to_str().unwrap().to_string();
        Self {
            checkout_dir,
            db_dir,
        }
    }

    // remove crates
    pub(crate) fn remove_crate(&self, crate_name: &str) {
        if crate_name.contains("-HEAD") {
            remove_crate(Path::new(&self.db_dir), crate_name);
        } else {
            remove_crate(Path::new(&self.checkout_dir), crate_name);
        }
        #[cfg(feature = "colored-output")]
        println!("{} {:?}", "Removed".red(), crate_name);
        #[cfg(feature = "non-colored-output")]
        println!("Removed {:?}", crate_name);
    }

    // Remove list of crates
    pub(crate) fn remove_crate_list(&self, crate_detail: &CrateDetail, list: &[String]) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find(crate_name, "GIT")
        }
        size_cleaned
    }

    // Remove all crate from git folder
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
        let mut size_cleaned = 0.0;

        if app.is_present("registry") {
            let git_subcommand = app.subcommand_matches("git").unwrap();
            if git_subcommand.is_present("include") {
                let value = git_subcommand.value_of("include").unwrap().to_string();
                cmd_include.push(value);
            }

            if git_subcommand.is_present("exclude") {
                let value = git_subcommand.value_of("exclude").unwrap().to_string();
                cmd_exclude.push(value);
            }
        }

        // Provide one time include crate list for all flag
        if app.is_present("include") {
            let value = app.value_of("include").unwrap().to_string();
            cmd_include.push(value);
        }

        // Provide one time exclude crate list for all flag
        if app.is_present("exclude") {
            let value = app.value_of("exclude").unwrap().to_string();
            cmd_exclude.push(value);
        }

        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        if cmd_include.contains(crate_name) || read_include.contains(crate_name) {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
        if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
        size_cleaned
    }
}

// preform remove operation
fn remove_crate(location: &Path, crate_name: &str) {
    for entry in fs::read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
        let crate_name = name[1];
        let rev_sha = name[0];
        if path.to_str().unwrap().contains(crate_name) {
            if rev_sha.contains("HEAD") {
                fs::remove_dir_all(&path).unwrap();
            } else {
                for rev in fs::read_dir(path).unwrap() {
                    let entry = rev.unwrap();
                    let path = entry.path();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    if file_name == rev_sha {
                        fs::remove_dir_all(&path).unwrap();
                    }
                }
            }
        }
    }
}
