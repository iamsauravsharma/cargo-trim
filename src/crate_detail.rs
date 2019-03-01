use std::collections::HashMap;

pub struct CrateDetail {
    crates_source: HashMap<String, u64>,
    crates_archive: HashMap<String, u64>,
}

impl CrateDetail {
    pub(crate) fn new() -> Self {
        Self {
            crates_source: HashMap::new(),
            crates_archive: HashMap::new(),
        }
    }

    pub(crate) fn add_crate_source(&mut self, crate_name: String, size: u64) {
        if let Some(val) = self.crates_source.get(&crate_name) {
            self.crates_source.insert(crate_name, size + val);
        } else {
            self.crates_source.insert(crate_name, size);
        }
    }

    pub(crate) fn add_crate_archive(&mut self, crate_name: String, size: u64) {
        if let Some(val) = self.crates_archive.get(&crate_name) {
            self.crates_archive.insert(crate_name, size + val);
        } else {
            self.crates_archive.insert(crate_name, size);
        }
    }

    pub(crate) fn find_size_source(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.crates_source.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_archive(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.crates_archive.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_all(&self, crate_name: &str) -> f64 {
        self.find_size_archive(crate_name) + self.find_size_source(crate_name)
    }
}
