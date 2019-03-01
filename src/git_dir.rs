use colored::*;
use std::{fs, path::Path};

pub struct GitDir {
    checkout_dir: String,
    db_dir: String,
}

impl GitDir {
    pub(super) fn new(checkout_dir: &Path, db_dir: &Path) -> Self {
        let checkout_dir = checkout_dir.to_str().unwrap().to_string();
        let db_dir = db_dir.to_str().unwrap().to_string();
        Self {
            checkout_dir,
            db_dir,
        }
    }

    pub(super) fn remove_crate(&self, crate_name: &str) {
        if crate_name.contains("-HEAD") {
            remove_crate(Path::new(&self.db_dir), crate_name);
        } else {
            remove_crate(Path::new(&self.checkout_dir), crate_name);
        }
        println!("{} {:?}", "removed".red(), crate_name);
    }
}

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
