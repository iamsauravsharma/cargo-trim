use std::{collections::HashMap, fs, path::Path};

use crate::utils::get_size;

// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    bin: HashMap<String, u64>,
    git_crates_source: HashMap<String, u64>,
    registry_crates_source: HashMap<String, u64>,
    git_crates_archive: HashMap<String, u64>,
    registry_crates_archive: HashMap<String, u64>,
}

impl CrateDetail {
    // return bin crates size information
    pub(crate) fn bin(&self) -> &HashMap<String, u64> {
        &self.bin
    }

    // return git crates source size information
    pub(crate) fn git_crates_source(&self) -> &HashMap<String, u64> {
        &self.git_crates_source
    }

    // return registry crates source size information
    pub(crate) fn registry_crates_source(&self) -> &HashMap<String, u64> {
        &self.registry_crates_source
    }

    // return git crates archive size information
    pub(crate) fn git_crates_archive(&self) -> &HashMap<String, u64> {
        &self.git_crates_archive
    }

    // return registry crates archive size information
    pub(crate) fn registry_crates_archive(&self) -> &HashMap<String, u64> {
        &self.registry_crates_archive
    }

    // add bin information to CrateDetail
    fn add_bin(&mut self, bin_name: String, size: u64) {
        self.bin.insert(bin_name, size);
    }

    // add git crate source information to CrateDetail
    fn add_git_crate_source(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.git_crates_source, crate_name, size)
    }

    // add registry crate source information to CrateDetail
    fn add_registry_crate_source(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.registry_crates_source, crate_name, size)
    }

    // add git crate archive information to CrateDetail
    fn add_git_crate_archive(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.git_crates_archive, crate_name, size)
    }

    // add registry crate archive information to CrateDetail
    fn add_registry_crate_archive(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.registry_crates_archive, crate_name, size)
    }

    // find size of certain git crate source in KB
    fn find_size_git_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_source, crate_name)
    }

    // find size of certain registry source in KB
    fn find_size_registry_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_source, crate_name)
    }

    // find size of certain git crate archive in KB
    fn find_size_git_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_archive, crate_name)
    }

    // find size of certain registry archive in KB

    fn find_size_registry_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_archive, crate_name)
    }

    // return certain git crate total size in KB
    pub(crate) fn find_size_git_all(&self, crate_name: &str) -> f64 {
        self.find_size_git_archive(crate_name) + self.find_size_git_source(crate_name)
    }

    // return certain registry crate total size in KB
    pub(crate) fn find_size_registry_all(&self, crate_name: &str) -> f64 {
        self.find_size_registry_archive(crate_name) + self.find_size_registry_source(crate_name)
    }

    // find crate size if location/title is given in KB
    pub(crate) fn find(&self, crate_name: &str, location: &str) -> f64 {
        if location.contains("REGISTRY") {
            self.find_size_registry_all(crate_name)
        } else if location.contains("GIT") {
            self.find_size_git_all(crate_name)
        } else {
            0.0
        }
    }

    // list installed bin
    pub(crate) fn get_installed_bin(&mut self, bin_dir: &Path) -> Vec<String> {
        let mut installed_bin = Vec::new();
        if bin_dir.exists() {
            for entry in fs::read_dir(bin_dir).expect("failed to read bin directory") {
                let entry = entry.unwrap().path();
                let bin_size = get_size(&entry).expect("failed to get size of bin directory");
                let file_name = entry
                    .file_name()
                    .expect("failed to get file name from bin directory");
                let bin_name = file_name.to_str().unwrap().to_string();
                self.add_bin(bin_name.to_owned(), bin_size);
                installed_bin.push(bin_name)
            }
        }
        installed_bin.sort();
        installed_bin
    }

    // list all installed registry crates
    pub(crate) fn get_installed_crate_registry(
        &mut self,
        src_dir: &Path,
        cache_dir: &Path,
    ) -> Vec<String> {
        let mut installed_crate_registry = Vec::new();
        // read src dir to get installed crate
        if src_dir.exists() {
            for entry in fs::read_dir(src_dir).expect("failed to read src directory") {
                let registry = entry.unwrap().path();
                for entry in fs::read_dir(registry).expect("failed to read registry folder") {
                    let entry = entry.unwrap().path();
                    let crate_size = get_size(&entry).expect("failed to get registry crate size");
                    let file_name = entry
                        .file_name()
                        .expect("failed to get file name form main entry");
                    let crate_name = file_name.to_str().unwrap();
                    self.add_registry_crate_source(crate_name.to_owned(), crate_size);
                    installed_crate_registry.push(crate_name.to_owned())
                }
            }
        }
        // read cache dir to get installed crate
        if cache_dir.exists() {
            for entry in fs::read_dir(cache_dir).expect("failed to read cache dir") {
                let registry = entry.unwrap().path();
                for entry in
                    fs::read_dir(registry).expect("failed to read cache dir registry folder")
                {
                    let entry = entry.unwrap().path();
                    let file_name = entry
                        .file_name()
                        .expect("failed to get file name from cache dir");
                    let crate_size = get_size(&entry).expect("failed to get size");
                    let crate_name = file_name.to_str().unwrap();
                    let split_name = crate_name.rsplitn(2, '.').collect::<Vec<&str>>();
                    self.add_registry_crate_archive(split_name[1].to_owned(), crate_size);
                    installed_crate_registry.push(split_name[1].to_owned());
                }
            }
        }
        installed_crate_registry.sort();
        installed_crate_registry.dedup();
        installed_crate_registry
    }

    // list all installed git crates
    pub(crate) fn get_installed_crate_git(
        &mut self,
        checkout_dir: &Path,
        db_dir: &Path,
    ) -> Vec<String> {
        let mut installed_crate_git = Vec::new();
        if checkout_dir.exists() {
            // read checkout dir to list crate name in form of crate_name-rev_sha
            for entry in fs::read_dir(checkout_dir).expect("failed to read checkout directory") {
                let entry = entry.unwrap().path();
                let path = entry.as_path();
                let file_path = path
                    .file_name()
                    .expect("failed to obtain checkout directory sub folder file name");
                for git_sha_entry in
                    fs::read_dir(path).expect("failed to read checkout dir sub folder")
                {
                    let git_sha_entry = git_sha_entry.unwrap().path();
                    let crate_size = get_size(&git_sha_entry).expect("failed to get folder size");
                    let git_sha_file_name =
                        git_sha_entry.file_name().expect("failed to get file name");
                    let git_sha = git_sha_file_name.to_str().unwrap();
                    let file_name = file_path.to_str().unwrap();
                    let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                    let full_name = format!("{}-{}", split_name[1], git_sha);
                    self.add_git_crate_archive(full_name.to_owned(), crate_size);
                    installed_crate_git.push(full_name)
                }
            }
        }
        // read a database directory to list a git crate in form of crate_name-HEAD
        if db_dir.exists() {
            for entry in fs::read_dir(db_dir).expect("failed to read db dir") {
                let entry = entry.unwrap().path();
                let crate_size = get_size(&entry).expect("failed to get size of db dir folders");
                let file_name = entry.file_name().expect("failed to get file name");
                let file_name = file_name.to_str().unwrap();
                let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let full_name = format!("{}-HEAD", split_name[1]);
                self.add_git_crate_source(full_name.to_owned(), crate_size);
                installed_crate_git.push(full_name);
            }
        }
        installed_crate_git.sort();
        installed_crate_git.dedup();
        installed_crate_git
    }
}

#[allow(clippy::cast_precision_loss)]
fn get_hashmap_crate_size(hashmap: &HashMap<String, u64>, crate_name: &str) -> f64 {
    hashmap
        .get(crate_name)
        .map_or(0.0, |size| (*size as f64) / 1000_f64.powi(2))
}

fn add_crate_to_hash_map(hashmap: &mut HashMap<String, u64>, crate_name: String, size: u64) {
    if let Some(crate_size) = hashmap.get_mut(&crate_name) {
        *crate_size += size;
    } else {
        hashmap.insert(crate_name, size);
    }
}

#[cfg(test)]
mod test {
    use super::{add_crate_to_hash_map, get_hashmap_crate_size};
    use std::collections::HashMap;
    #[test]
    fn test_get_hashmap_crate_size() {
        let mut hashmap_content = HashMap::new();
        hashmap_content.insert("sample_crate".to_string(), 1000u64);
        hashmap_content.insert("sample_crate_2".to_string(), 20u64);

        assert_eq!(
            get_hashmap_crate_size(&hashmap_content, "sample_crate_2"),
            0.00002
        );
        assert_eq!(
            get_hashmap_crate_size(&hashmap_content, "sample_crate_3"),
            0.0
        );
    }
    #[test]
    fn test_add_crate_to_hashmap() {
        let mut hashmap_content = HashMap::new();
        hashmap_content.insert("sample_crate".to_string(), 1000u64);
        hashmap_content.insert("sample_crate_2".to_string(), 20u64);
        add_crate_to_hash_map(&mut hashmap_content, "sample_crate_2".to_string(), 3000);
        add_crate_to_hash_map(&mut hashmap_content, "sample_crate_3".to_string(), 2500);

        let mut another_hashmap = HashMap::new();
        another_hashmap.insert("sample_crate".to_string(), 1000u64);
        another_hashmap.insert("sample_crate_2".to_string(), 3020u64);
        another_hashmap.insert("sample_crate_3".to_string(), 2500u64);

        assert_eq!(hashmap_content, another_hashmap);
    }
}
