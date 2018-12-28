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
        remove_crate(Path::new(&self.checkout_dir), crate_name);
        remove_crate(Path::new(&self.db_dir), crate_name);
        println!("Removed {:?}", crate_name);
    }
}

fn remove_crate(location: &Path, crate_name: &str) {
    for entry in fs::read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.to_str().unwrap().contains(crate_name) {
            fs::remove_dir_all(path).unwrap();
        }
    }
}
