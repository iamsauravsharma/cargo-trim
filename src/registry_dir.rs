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
