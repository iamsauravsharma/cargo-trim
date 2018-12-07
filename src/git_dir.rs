use std::{fs, path::Path};

// Stores .cargo/registry cache & src information
pub struct GitDir {
    git_cache_dir: String,
    git_src_dir: String,
}

impl GitDir {
    // Create new GitDir
    pub(super) fn new(cache_dir: &Path, src_dir: &Path) -> GitDir {
        let git_cache_dir = open_github_folder(&cache_dir).unwrap();
        let git_src_dir = open_github_folder(&src_dir).unwrap();
        GitDir {
            git_cache_dir,
            git_src_dir,
        }
    }

    // Remove crate from src & cache directory
    pub(super) fn remove_crate(&self, crate_name: &str) {
        remove_crate(Path::new(&self.git_cache_dir), crate_name);
        remove_crate(Path::new(&self.git_src_dir), crate_name);
        println!("Removed {:?}", crate_name);
    }

    // Get out src_dir path
    pub(super) fn src(&self) -> &String {
        &self.git_src_dir
    }
}

// Use to open github folder present inside src and cache folder
fn open_github_folder(path: &Path) -> Option<String> {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();
        if path.contains("github.com") {
            return Some(path.to_string());
        }
    }
    None
}

// Remove crates which name is provided to delete
fn remove_crate(path: &Path, value: &str) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.to_str().unwrap().contains(value) {
            if path.is_file() {
                fs::remove_file(path).unwrap();
            } else if path.is_dir() {
                fs::remove_dir_all(path).unwrap();
            }
        } else if path.is_dir() {
            remove_crate(&path, value);
        }
    }
}
