use crate::{ConfigFile, CrateDetail};
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
        crate_name: &str,
        crate_detail: &CrateDetail,
    ) -> f64 {
        let crate_name = &crate_name.to_string();
        let mut size_cleaned = 0.0;
        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        let crate_split = crate_name.rsplitn(2, '-');
        let mut simple_name = String::new();
        for (i, val) in crate_split.enumerate() {
            if i == 1 {
                simple_name = val.to_string()
            }
        }
        if read_include.contains(crate_name) || read_include.contains(&simple_name) {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
        if !read_exclude.contains(crate_name) && !read_exclude.contains(&simple_name) {
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
