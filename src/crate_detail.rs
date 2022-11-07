use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fs;
use std::hash::Hash;
use std::path::Path;

use anyhow::{Context, Result};
use semver::Version;

use crate::utils::{get_size, split_name_version};

#[derive(Debug, Clone)]
pub(crate) struct CrateMetaData {
    name: String,
    version: Option<Version>,
    size: u64,
    source: Option<String>,
}

impl CrateMetaData {
    pub(crate) fn new(
        name: String,
        version: Option<Version>,
        size: u64,
        source: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            size,
            source,
        }
    }

    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    pub(crate) fn version(&self) -> &Option<Version> {
        &self.version
    }

    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn source(&self) -> &Option<String> {
        &self.source
    }
}

impl PartialOrd for CrateMetaData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CrateMetaData {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.name.cmp(&other.name) {
            Ordering::Equal => {
                match self.source.cmp(&other.source) {
                    Ordering::Equal => {
                        match self.version.cmp(&other.version) {
                            Ordering::Equal => self.size.cmp(&other.size),
                            ord => ord,
                        }
                    }
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl PartialEq for CrateMetaData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version && self.source == other.source
    }
}

impl Eq for CrateMetaData {}

impl Hash for CrateMetaData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
        self.source.hash(state);
    }
}

/// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    source_info: HashMap<String, String>,
    bin: HashSet<CrateMetaData>,
    git_crates_source: HashSet<CrateMetaData>,
    registry_crates_source: HashSet<CrateMetaData>,
    git_crates_archive: HashSet<CrateMetaData>,
    registry_crates_archive: HashSet<CrateMetaData>,
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

    /// return bin crates metadata
    pub(crate) fn bin(&self) -> &HashSet<CrateMetaData> {
        &self.bin
    }

    /// return git crates source
    pub(crate) fn git_crates_source(&self) -> &HashSet<CrateMetaData> {
        &self.git_crates_source
    }

    /// return registry crates source metadata
    pub(crate) fn registry_crates_source(&self) -> &HashSet<CrateMetaData> {
        &self.registry_crates_source
    }

    /// return git crates archive metadata
    pub(crate) fn git_crates_archive(&self) -> &HashSet<CrateMetaData> {
        &self.git_crates_archive
    }

    /// return registry crates archive metadata
    pub(crate) fn registry_crates_archive(&self) -> &HashSet<CrateMetaData> {
        &self.registry_crates_archive
    }

    /// add bin information to crate detail
    fn add_bin(&mut self, bin_metadata: &CrateMetaData) {
        self.bin.insert(bin_metadata.clone());
    }

    /// add git crate source information to crate detail
    fn add_git_crate_source(&mut self, crate_metadata: &CrateMetaData) {
        self.git_crates_source.insert(crate_metadata.clone());
    }

    /// add registry crate source information to crate detail
    fn add_registry_crate_source(&mut self, crate_metadata: &CrateMetaData) {
        self.registry_crates_source.insert(crate_metadata.clone());
    }

    /// add git crate archive information to crate detail
    fn add_git_crate_archive(&mut self, crate_metadata: &CrateMetaData) {
        self.git_crates_archive.insert(crate_metadata.clone());
    }

    /// add registry crate archive information to crate detail
    fn add_registry_crate_archive(&mut self, crate_metadata: &CrateMetaData) {
        self.registry_crates_archive.insert(crate_metadata.clone());
    }

    /// list installed bin
    pub(crate) fn list_installed_bin(&mut self, bin_dir: &Path) -> Result<Vec<CrateMetaData>> {
        let mut installed_bin = Vec::new();
        if bin_dir.exists() {
            for entry in fs::read_dir(bin_dir).context("failed to read bin directory")? {
                let entry = entry?.path();
                let bin_size = get_size(&entry).context("failed to get size of bin directory")?;
                let file_name = entry
                    .file_name()
                    .context("failed to get file name from bin directory")?;
                let bin_name = file_name
                    .to_str()
                    .context("failed to convert file name of bin to str")?
                    .to_string();
                let bin_metadata = CrateMetaData {
                    name: bin_name,
                    version: None,
                    size: bin_size,
                    source: None,
                };
                self.add_bin(&bin_metadata);
                installed_bin.push(bin_metadata);
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
    ) -> Result<Vec<CrateMetaData>> {
        let mut installed_crate_registry = HashSet::new();
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
                        .context("failed to get file name from main entry")?;
                    let crate_name = file_name
                        .to_str()
                        .context("Failed to convert crate file name to str")?;
                    let (name, version) = split_name_version(crate_name)?;
                    let crate_metadata = CrateMetaData {
                        name,
                        version: Some(version),
                        size: crate_size,
                        source: Some(source.clone()),
                    };
                    self.add_registry_crate_source(&crate_metadata);
                    update_crate_list(&mut installed_crate_registry, &crate_metadata)?;
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
                    let crate_name = file_name
                        .to_str()
                        .context("Failed to convert crate file name to str")?;
                    let (name, version) = split_name_version(crate_name)?;
                    let crate_metadata = CrateMetaData {
                        name,
                        version: Some(version),
                        size: crate_size,
                        source: Some(source.clone()),
                    };
                    self.add_registry_crate_archive(&crate_metadata);
                    update_crate_list(&mut installed_crate_registry, &crate_metadata)?;
                }
            }
        }
        let mut installed_crates = Vec::new();
        for crate_metadata in installed_crate_registry {
            installed_crates.push(crate_metadata);
        }
        installed_crates.sort();
        Ok(installed_crates)
    }

    /// list all installed git crates
    pub(crate) fn list_installed_crate_git(
        &mut self,
        checkout_dir: &Path,
        db_dir: &Path,
    ) -> Result<Vec<CrateMetaData>> {
        let mut installed_crate_git = HashSet::new();
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
                    let git_sha = git_sha_file_name
                        .to_str()
                        .context("Failed to convert git sha name to str")?;
                    let file_name = file_path
                        .to_str()
                        .context("Failed to convert file path file name to str")?;
                    let full_name = format!("{file_name}-{git_sha}");
                    let crate_metadata = CrateMetaData {
                        name: full_name,
                        version: None,
                        size: crate_size,
                        source: Some(source.clone()),
                    };
                    self.add_git_crate_archive(&crate_metadata);
                    update_crate_list(&mut installed_crate_git, &crate_metadata)?;
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
                let file_name = file_name
                    .to_str()
                    .context("Failed to convert db dir file name to str")?;
                let full_name = format!("{file_name}-HEAD");
                let crate_metadata = CrateMetaData {
                    name: full_name,
                    version: None,
                    size: crate_size,
                    source: Some(source),
                };
                self.add_git_crate_source(&crate_metadata);
                update_crate_list(&mut installed_crate_git, &crate_metadata)?;
            }
        }
        let mut installed_crates = Vec::new();
        for crate_metadata in installed_crate_git {
            installed_crates.push(crate_metadata);
        }
        installed_crates.sort();
        Ok(installed_crates)
    }
}

fn update_crate_list(
    hash_set: &mut HashSet<CrateMetaData>,
    temp_crate_metadata: &CrateMetaData,
) -> Result<()> {
    let meta_data_exists = hash_set.get(temp_crate_metadata).is_some();
    let mut current_size = temp_crate_metadata.size;
    if meta_data_exists {
        current_size += hash_set
            .get(temp_crate_metadata)
            .context("failed to get metadata from hash set")?
            .size;
    }
    hash_set.remove(temp_crate_metadata);
    hash_set.insert(CrateMetaData {
        size: current_size,
        ..temp_crate_metadata.clone()
    });
    Ok(())
}
