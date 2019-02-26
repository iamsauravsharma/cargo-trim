use fs_extra::dir::get_size;
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

    pub(super) fn remove_crate(&self, crate_name: &str) -> f64 {
        let mut total_size_saved: u64 = 0;
        if crate_name.contains("-HEAD") {
            total_size_saved += remove_crate(Path::new(&self.db_dir), crate_name);
        } else {
            total_size_saved += remove_crate(Path::new(&self.checkout_dir), crate_name);
        }
        println!("Removed {:?}", crate_name);
        (total_size_saved as f64) / (1024f64.powf(2.0))
    }
}

fn remove_crate(location: &Path, crate_name: &str) -> u64 {
    let mut file_size: u64 = 0;
    for entry in fs::read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
        let crate_name = name[1];
        let rev_sha = name[0];
        if path.to_str().unwrap().contains(crate_name) {
            if !rev_sha.contains("HEAD") {
                for rev in fs::read_dir(path).unwrap() {
                    let entry = rev.unwrap();
                    let path = entry.path();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    if file_name == rev_sha {
                        file_size += get_size(&path).unwrap();
                        fs::remove_dir_all(&path).unwrap();
                    }
                }
            } else {
                file_size += get_size(&path).unwrap();
                fs::remove_dir_all(&path).unwrap();
            }
        }
    }
    file_size
}
