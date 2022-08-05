use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::utils::get_size;

#[derive(Debug)]
pub(crate) struct CrateMetaData {
    size: u64,
    source: Option<String>,
}

impl CrateMetaData {
    pub(crate) fn size(&self) -> u64 {
        self.size
    }
}

impl PartialEq for CrateMetaData {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.source == other.source
    }
}

/// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    source_info: HashMap<String, String>,
    bin: HashMap<String, CrateMetaData>,
    git_crates_source: HashMap<String, CrateMetaData>,
    registry_crates_source: HashMap<String, CrateMetaData>,
    git_crates_archive: HashMap<String, CrateMetaData>,
    registry_crates_archive: HashMap<String, CrateMetaData>,
}

impl CrateDetail {
    /// Crate new index info
    pub(crate) fn new(index_dir: &Path, db_dir: &Path) -> Result<Self> {
        let mut source_info = HashMap::new();
        if index_dir.exists() {
            for entry in fs::read_dir(index_dir)? {
                let registry_dir = entry?.path();
                let registry_file_name = registry_dir
                    .file_name()
                    .context("Failed to get file name of registry dir")?
                    .to_str()
                    .context("Failed to convert osstr to str")?;
                let mut fetch_head_file = registry_dir.clone();
                fetch_head_file.push(".git");
                fetch_head_file.push("FETCH_HEAD");
                let content = fs::read_to_string(fetch_head_file)
                    .context("Failed to read FETCH_HEAD file")?;
                let url_path = content
                    .split_whitespace()
                    .last()
                    .context("Failed to get url part from content")?;
                source_info.insert(registry_file_name.to_string(), url_path.to_string());
            }
        }
        if db_dir.exists() {
            for entry in fs::read_dir(db_dir)? {
                let git_dir = entry?.path();
                let git_file_name = git_dir
                    .file_name()
                    .context("Failed to get file name of git dir")?
                    .to_str()
                    .context("Failed to convert osstr to str")?;
                let mut fetch_head_file = git_dir.clone();
                fetch_head_file.push("FETCH_HEAD");
                let content = fs::read_to_string(fetch_head_file)
                    .context("Failed to read FETCH_HEAD file")?;
                let url_path = content
                    .split_whitespace()
                    .last()
                    .context("Failed to get url part from content")?;
                source_info.insert(git_file_name.to_string(), url_path.to_string());
            }
        }
        Ok(Self {
            source_info,
            ..Default::default()
        })
    }

    /// Get source value from path
    pub(crate) fn source_url_from_path(&self, path: &Path) -> Result<String> {
        let file_name = path
            .file_name()
            .context("Failed to get file name of path")?
            .to_str()
            .context("Failed to convert osstr to str")?;
        Ok(self
            .source_info
            .get(file_name)
            .context("Failed to get url for path")?
            .to_string())
    }

    /// return bin crates size information
    pub(crate) fn bin(&self) -> &HashMap<String, CrateMetaData> {
        &self.bin
    }

    /// return git crates source size information
    pub(crate) fn git_crates_source(&self) -> &HashMap<String, CrateMetaData> {
        &self.git_crates_source
    }

    /// return registry crates source size information
    pub(crate) fn registry_crates_source(&self) -> &HashMap<String, CrateMetaData> {
        &self.registry_crates_source
    }

    /// return git crates archive size information
    pub(crate) fn git_crates_archive(&self) -> &HashMap<String, CrateMetaData> {
        &self.git_crates_archive
    }

    /// return registry crates archive size information
    pub(crate) fn registry_crates_archive(&self) -> &HashMap<String, CrateMetaData> {
        &self.registry_crates_archive
    }

    /// add bin information to crate detail
    fn add_bin(&mut self, bin_name: String, size: u64) {
        self.bin
            .insert(bin_name, CrateMetaData { size, source: None });
    }

    /// add git crate source information to crate detail
    fn add_git_crate_source(&mut self, crate_name: String, size: u64, source: String) {
        add_crate_to_hash_map(&mut self.git_crates_source, crate_name, size, source);
    }

    /// add registry crate source information to crate detail
    fn add_registry_crate_source(&mut self, crate_name: String, size: u64, source: String) {
        add_crate_to_hash_map(&mut self.registry_crates_source, crate_name, size, source);
    }

    /// add git crate archive information to crate detail
    fn add_git_crate_archive(&mut self, crate_name: String, size: u64, source: String) {
        add_crate_to_hash_map(&mut self.git_crates_archive, crate_name, size, source);
    }

    /// add registry crate archive information to crate detail
    fn add_registry_crate_archive(&mut self, crate_name: String, size: u64, source: String) {
        add_crate_to_hash_map(&mut self.registry_crates_archive, crate_name, size, source);
    }

    /// find size of certain git crate source in KB
    fn find_size_git_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_source, crate_name)
    }

    /// find size of certain registry source in KB
    fn find_size_registry_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_source, crate_name)
    }

    /// find size of certain git crate archive in KB
    fn find_size_git_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_archive, crate_name)
    }

    /// find size of certain registry archive in KB
    fn find_size_registry_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_archive, crate_name)
    }

    /// return certain git crate total size in KB
    pub(crate) fn find_size_git_all(&self, crate_name: &str) -> f64 {
        self.find_size_git_archive(crate_name) + self.find_size_git_source(crate_name)
    }

    /// return certain registry crate total size in KB
    pub(crate) fn find_size_registry_all(&self, crate_name: &str) -> f64 {
        self.find_size_registry_archive(crate_name) + self.find_size_registry_source(crate_name)
    }

    /// find crate size if location/title is given in KB
    pub(crate) fn find(&self, crate_name: &str, location: &str) -> f64 {
        if location.contains("REGISTRY") {
            self.find_size_registry_all(crate_name)
        } else if location.contains("GIT") {
            self.find_size_git_all(crate_name)
        } else {
            0.0
        }
    }

    /// list installed bin
    pub(crate) fn list_installed_bin(&mut self, bin_dir: &Path) -> Result<Vec<String>> {
        let mut installed_bin = Vec::new();
        if bin_dir.exists() {
            for entry in fs::read_dir(bin_dir).context("failed to read bin directory")? {
                let entry = entry?.path();
                let bin_size = get_size(&entry).context("failed to get size of bin directory")?;
                let file_name = entry
                    .file_name()
                    .context("failed to get file name from bin directory")?;
                let bin_name = file_name.to_str().unwrap().to_string();
                self.add_bin(bin_name.clone(), bin_size);
                installed_bin.push(bin_name);
            }
        }
        installed_bin.sort();
        Ok(installed_bin)
    }

    /// list all installed registry crates
    pub(crate) fn list_installed_crate_registry(
        &mut self,
        src_dir: &Path,
        cache_dir: &Path,
    ) -> Result<Vec<String>> {
        let mut installed_crate_registry = Vec::new();
        // read src dir to get installed crate
        if src_dir.exists() {
            for entry in fs::read_dir(src_dir).context("failed to read src directory")? {
                let registry = entry?.path();
                let source = self.source_url_from_path(&registry)?;
                for entry in fs::read_dir(registry).context("failed to read registry folder")? {
                    let entry = entry?.path();
                    let crate_size =
                        get_size(&entry).context("failed to get registry crate size")?;
                    let file_name = entry
                        .file_name()
                        .context("failed to get file name form main entry")?;
                    let crate_name = file_name.to_str().unwrap();
                    self.add_registry_crate_source(
                        crate_name.to_owned(),
                        crate_size,
                        source.clone(),
                    );
                    installed_crate_registry.push(crate_name.to_owned());
                }
            }
        }
        // read cache dir to get installed crate
        if cache_dir.exists() {
            for entry in fs::read_dir(cache_dir).context("failed to read cache dir")? {
                let registry = entry?.path();
                let source = self.source_url_from_path(&registry)?;
                for entry in
                    fs::read_dir(registry).context("failed to read cache dir registry folder")?
                {
                    let entry = entry?.path();
                    let file_name = entry
                        .file_name()
                        .context("failed to get file name from cache dir")?;
                    let crate_size = get_size(&entry).context("failed to get size")?;
                    let crate_name = file_name.to_str().unwrap();
                    let split_name = crate_name.rsplitn(2, '.').collect::<Vec<&str>>();
                    self.add_registry_crate_archive(
                        split_name[1].to_owned(),
                        crate_size,
                        source.clone(),
                    );
                    installed_crate_registry.push(split_name[1].to_owned());
                }
            }
        }
        installed_crate_registry.sort();
        installed_crate_registry.dedup();
        Ok(installed_crate_registry)
    }

    /// list all installed git crates
    pub(crate) fn list_installed_crate_git(
        &mut self,
        checkout_dir: &Path,
        db_dir: &Path,
    ) -> Result<Vec<String>> {
        let mut installed_crate_git = Vec::new();
        if checkout_dir.exists() {
            // read checkout dir to list crate name in form of crate_name-rev_sha
            for entry in fs::read_dir(checkout_dir).context("failed to read checkout directory")? {
                let entry = entry?.path();
                let source = self.source_url_from_path(&entry)?;
                let file_path = entry
                    .file_name()
                    .context("failed to obtain checkout directory sub folder file name")?;
                for git_sha_entry in
                    fs::read_dir(&entry).context("failed to read checkout dir sub folder")?
                {
                    let git_sha_entry = git_sha_entry?.path();
                    let crate_size =
                        get_size(&git_sha_entry).context("failed to get folder size")?;
                    let git_sha_file_name = git_sha_entry
                        .file_name()
                        .context("failed to get file name")?;
                    let git_sha = git_sha_file_name.to_str().unwrap();
                    let file_name = file_path.to_str().unwrap();
                    let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                    let full_name = format!("{}-{}", split_name[1], git_sha);
                    self.add_git_crate_archive(full_name.clone(), crate_size, source.clone());
                    installed_crate_git.push(full_name);
                }
            }
        }
        // read a database directory to list a git crate in form of crate_name-HEAD
        if db_dir.exists() {
            for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                let entry = entry?.path();
                let source = self.source_url_from_path(&entry)?;
                let crate_size =
                    get_size(&entry).context("failed to get size of db dir folders")?;
                let file_name = entry.file_name().context("failed to get file name")?;
                let file_name = file_name.to_str().unwrap();
                let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let full_name = format!("{}-HEAD", split_name[1]);
                self.add_git_crate_source(full_name.clone(), crate_size, source.clone());
                installed_crate_git.push(full_name);
            }
        }
        installed_crate_git.sort();
        installed_crate_git.dedup();
        Ok(installed_crate_git)
    }
}

/// Convert stored bytes size to KB and return f64 for crate from hashmap
#[allow(clippy::cast_precision_loss)]
fn get_hashmap_crate_size(hashmap: &HashMap<String, CrateMetaData>, crate_name: &str) -> f64 {
    hashmap
        .get(crate_name)
        .map_or(0.0, |info| (info.size as f64) / 1000_f64.powi(2))
}

fn add_crate_to_hash_map(
    hashmap: &mut HashMap<String, CrateMetaData>,
    crate_name: String,
    size: u64,
    source: String,
) {
    if let Some(info) = hashmap.get_mut(&crate_name) {
        info.size += size;
    } else {
        hashmap.insert(
            crate_name,
            CrateMetaData {
                size,
                source: Some(source),
            },
        );
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{add_crate_to_hash_map, get_hashmap_crate_size};
    use crate::crate_detail::CrateMetaData;
    #[test]
    fn test_get_hashmap_crate_size() {
        let mut hashmap_content = HashMap::new();
        hashmap_content.insert(
            "sample_crate".to_string(),
            CrateMetaData {
                size: 1000,
                source: Some("test".to_string()),
            },
        );
        hashmap_content.insert(
            "sample_crate_2".to_string(),
            CrateMetaData {
                size: 20,
                source: Some("test".to_string()),
            },
        );

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
        hashmap_content.insert(
            "sample_crate".to_string(),
            CrateMetaData {
                size: 10000,
                source: Some("test".to_string()),
            },
        );
        hashmap_content.insert(
            "sample_crate_2".to_string(),
            CrateMetaData {
                size: 20,
                source: Some("test".to_string()),
            },
        );
        add_crate_to_hash_map(
            &mut hashmap_content,
            "sample_crate_2".to_string(),
            3000,
            "test".to_string(),
        );
        add_crate_to_hash_map(
            &mut hashmap_content,
            "sample_crate_3".to_string(),
            2500,
            "test".to_string(),
        );

        let mut another_hashmap = HashMap::new();
        another_hashmap.insert(
            "sample_crate".to_string(),
            CrateMetaData {
                size: 10000,
                source: Some("test".to_string()),
            },
        );
        another_hashmap.insert(
            "sample_crate_2".to_string(),
            CrateMetaData {
                size: 3020,
                source: Some("test".to_string()),
            },
        );
        another_hashmap.insert(
            "sample_crate_3".to_string(),
            CrateMetaData {
                size: 2500,
                source: Some("test".to_string()),
            },
        );

        assert_eq!(hashmap_content, another_hashmap);
    }
}
