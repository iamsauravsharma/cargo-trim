use std::collections::HashMap;

pub struct CrateDetail {
    bin: HashMap<String, u64>,
    git_crates_source: HashMap<String, u64>,
    registry_crates_source: HashMap<String, u64>,
    git_crates_archive: HashMap<String, u64>,
    registry_crates_archive: HashMap<String, u64>,
}

impl CrateDetail {
    pub(crate) fn new() -> Self {
        Self {
            bin: HashMap::new(),
            git_crates_source: HashMap::new(),
            registry_crates_source: HashMap::new(),
            git_crates_archive: HashMap::new(),
            registry_crates_archive: HashMap::new(),
        }
    }

    pub(crate) fn bin(&self) -> HashMap<String, u64> {
        self.bin.to_owned()
    }

    pub(crate) fn git_crates_source(&self) -> HashMap<String, u64> {
        self.git_crates_source.to_owned()
    }

    pub(crate) fn registry_crates_source(&self) -> HashMap<String, u64> {
        self.registry_crates_source.to_owned()
    }

    pub(crate) fn git_crates_archive(&self) -> HashMap<String, u64> {
        self.git_crates_archive.to_owned()
    }

    pub(crate) fn registry_crates_archive(&self) -> HashMap<String, u64> {
        self.registry_crates_archive.to_owned()
    }

    pub(crate) fn add_bin(&mut self, bin_name: String, size: u64) {
        self.bin.insert(bin_name, size);
    }

    pub(crate) fn add_git_crate_source(&mut self, crate_name: String, size: u64) {
        self.git_crates_source.insert(crate_name, size);
    }

    pub(crate) fn add_registry_crate_source(&mut self, crate_name: String, size: u64) {
        self.registry_crates_source.insert(crate_name, size);
    }

    pub(crate) fn add_git_crate_archive(&mut self, crate_name: String, size: u64) {
        self.git_crates_archive.insert(crate_name, size);
    }

    pub(crate) fn add_registry_crate_archive(&mut self, crate_name: String, size: u64) {
        self.registry_crates_archive.insert(crate_name, size);
    }

    pub(crate) fn find_size_git_source(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.git_crates_source.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_registry_source(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.registry_crates_source.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_git_archive(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.git_crates_archive.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_registry_archive(&self, crate_name: &str) -> f64 {
        if let Some(size) = self.registry_crates_archive.get(crate_name) {
            (*size as f64) / 1024_f64.powf(2.0)
        } else {
            0.0
        }
    }

    pub(crate) fn find_size_git_all(&self, crate_name: &str) -> f64 {
        self.find_size_git_archive(crate_name) + self.find_size_git_source(crate_name)
    }

    pub(crate) fn find_size_registry_all(&self, crate_name: &str) -> f64 {
        self.find_size_registry_archive(crate_name) + self.find_size_registry_source(crate_name)
    }
}
