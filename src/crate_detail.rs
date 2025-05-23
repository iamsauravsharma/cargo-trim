use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use url::Url;

use crate::utils::{get_size, split_name_version};

#[derive(Debug, Clone)]
pub(crate) struct CrateMetaData {
    name: String,
    version: Option<Version>,
    size: u64,
    source: Option<String>,
    path: Option<PathBuf>,
}

impl CrateMetaData {
    pub(crate) fn new(name: String, version: Option<Version>, source: Option<String>) -> Self {
        Self {
            name,
            version,
            size: 0,
            source,
            path: None,
        }
    }

    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    pub(crate) fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }

    pub(crate) fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
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
                match self.version.cmp(&other.version) {
                    Ordering::Equal => self.source.cmp(&other.source),
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

#[derive(Deserialize)]
struct IndexConfig {
    dl: Url,
    api: Option<Url>,
}

/// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    source_infos: HashMap<String, Url>,
    bin: HashSet<CrateMetaData>,
    git_crates_source: HashSet<CrateMetaData>,
    registry_crates_source: HashSet<CrateMetaData>,
    git_crates_archive: HashSet<CrateMetaData>,
    registry_crates_archive: HashSet<CrateMetaData>,
}

impl CrateDetail {
    /// Crate new index info
    pub(crate) fn new(index_dir: &Path, db_dir: &Path) -> Result<Self> {
        let mut source_infos = HashMap::new();
        if index_dir.exists() && index_dir.is_dir() {
            for entry in fs::read_dir(index_dir)? {
                let registry_dir = entry?.path();
                let registry_file_name = registry_dir
                    .file_name()
                    .context("failed to get file name of registry dir")?
                    .to_str()
                    .context("failed to convert OSstr to str")?;

                // file for git based registry
                let mut fetch_head_file = registry_dir.clone();
                fetch_head_file.push(".git");
                fetch_head_file.push("FETCH_HEAD");

                // file for http based registry
                let mut config_file = registry_dir.clone();
                config_file.push("config.json");

                // Check if fetch head file exists if it exists than index is old registry based
                // index instead of new sparse based
                if fetch_head_file.exists() {
                    let content = fs::read_to_string(fetch_head_file)
                        .context("failed to read FETCH_HEAD file")?;
                    let url_path = content
                        .split_whitespace()
                        .last()
                        .context("failed to get url part from content")?;
                    source_infos.insert(
                        registry_file_name.to_string(),
                        Url::from_str(url_path).context("fail FETCH_HEAD url conversion")?,
                    );
                // Else if config file exists it is based on sparse registry
                } else if config_file.exists() {
                    let domain = registry_file_name
                        .rsplitn(2, '-')
                        .last()
                        .context("failed to get url for sparse registry")?;

                    let content = fs::read_to_string(config_file)
                        .context("failed to read config.json file")?;
                    let json: IndexConfig = serde_json::from_str(&content)?;
                    // First use api url if api url exists else use dl url for determining scheme
                    // since file name have no information about scheme
                    let scheme_url = json.api.unwrap_or(json.dl);
                    let scheme = scheme_url.scheme();
                    let url = Url::from_str(&format!("{scheme}://{domain}"))
                        .context("failed sparse registry index url")?;
                    source_infos.insert(registry_file_name.to_string(), url);
                }
            }
        }
        if db_dir.exists() && db_dir.is_dir() {
            for entry in fs::read_dir(db_dir)? {
                let git_dir = entry?.path();
                let git_file_name = git_dir
                    .file_name()
                    .context("failed to get file name of git dir")?
                    .to_str()
                    .context("failed to convert osstr to str")?;
                let mut fetch_head_file = git_dir.clone();
                fetch_head_file.push("FETCH_HEAD");
                let content = fs::read_to_string(fetch_head_file)
                    .context("failed to read FETCH_HEAD file")?;
                let url_path = content
                    .split_whitespace()
                    .last()
                    .context("failed to get url part from content")?;
                source_infos.insert(
                    git_file_name.to_string(),
                    Url::from_str(url_path).context("failed to convert db dir FETCH_HEAD")?,
                );
            }
        }
        Ok(Self {
            source_infos,
            ..Default::default()
        })
    }

    /// Get index name from url
    pub(crate) fn index_names_from_url(&self, url: &Url) -> Vec<String> {
        self.source_infos
            .iter()
            .filter_map(|(key, val)| {
                if val == url {
                    Some(key.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get source infos
    pub(crate) fn source_infos(&self) -> &HashMap<String, Url> {
        &self.source_infos
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
        if bin_dir.exists() && bin_dir.is_dir() {
            for entry in fs::read_dir(bin_dir).context("failed to read bin directory")? {
                let entry_path = entry?.path();
                let bin_size =
                    get_size(&entry_path).context("failed to get size of bin directory")?;
                let file_name = entry_path
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
                    path: None,
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
        if src_dir.exists() && src_dir.is_dir() {
            for entry in fs::read_dir(src_dir).context("failed to read src directory")? {
                let registry = entry?.path();
                if registry.is_dir() {
                    for dir_entry in
                        fs::read_dir(&registry).context("failed to read registry folder")?
                    {
                        let dir_entry_path = dir_entry?.path();
                        let crate_size = get_size(&dir_entry_path)
                            .context("failed to get registry crate size")?;
                        let file_name = dir_entry_path
                            .file_name()
                            .context("failed to get file name from main entry")?;
                        let crate_name = file_name
                            .to_str()
                            .context("failed to convert crate file name to str")?;
                        let (name, version) = split_name_version(crate_name)?;
                        let crate_metadata = CrateMetaData {
                            name,
                            version: Some(version),
                            size: crate_size,
                            source: registry
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(ToString::to_string),
                            path: Some(dir_entry_path),
                        };
                        self.add_registry_crate_source(&crate_metadata);
                        update_crate_list(&mut installed_crate_registry, &crate_metadata)?;
                    }
                }
            }
        }
        // read cache dir to get installed crate
        if cache_dir.exists() && cache_dir.is_dir() {
            for entry in fs::read_dir(cache_dir).context("failed to read cache dir")? {
                let registry = entry?.path();
                if registry.is_dir() {
                    for dir_entry in fs::read_dir(&registry)
                        .context("failed to read cache dir registry folder")?
                    {
                        let dir_entry_path = dir_entry?.path();
                        let file_name = dir_entry_path
                            .file_name()
                            .context("failed to get file name from cache dir")?;
                        let crate_size = get_size(&dir_entry_path).context("failed to get size")?;
                        let crate_name = file_name
                            .to_str()
                            .context("failed to convert crate file name to str")?;
                        let (name, version) = split_name_version(crate_name)?;
                        let crate_metadata = CrateMetaData {
                            name,
                            version: Some(version),
                            size: crate_size,
                            source: registry
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(ToString::to_string),
                            path: Some(dir_entry_path),
                        };
                        self.add_registry_crate_archive(&crate_metadata);
                        update_crate_list(&mut installed_crate_registry, &crate_metadata)?;
                    }
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
        if checkout_dir.exists() && checkout_dir.is_dir() {
            // read checkout dir to list crate name in form of crate_name-rev_sha
            for entry in fs::read_dir(checkout_dir).context("failed to read checkout directory")? {
                let entry_path = entry?.path();
                if entry_path.is_dir() {
                    let file_path = entry_path
                        .file_name()
                        .context("failed to obtain checkout directory sub folder file name")?;
                    for git_sha_entry in fs::read_dir(&entry_path)
                        .context("failed to read checkout dir sub folder")?
                    {
                        let git_sha_entry_path = git_sha_entry?.path();
                        let crate_size =
                            get_size(&git_sha_entry_path).context("failed to get folder size")?;
                        let git_sha_file_name = git_sha_entry_path
                            .file_name()
                            .context("failed to get file name")?;
                        let git_sha = git_sha_file_name
                            .to_str()
                            .context("failed to convert git sha name to str")?;
                        let file_name = file_path
                            .to_str()
                            .context("failed to convert file path file name to str")?;
                        let splitted_file_name = file_name.rsplitn(2, '-').collect::<Vec<_>>();
                        let crate_name_initial = splitted_file_name[1];
                        let full_name = format!("{crate_name_initial}-{git_sha}");
                        let crate_metadata = CrateMetaData {
                            name: full_name,
                            version: None,
                            size: crate_size,
                            source: entry_path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(ToString::to_string),
                            path: Some(git_sha_entry_path),
                        };
                        self.add_git_crate_archive(&crate_metadata);
                        update_crate_list(&mut installed_crate_git, &crate_metadata)?;
                    }
                }
            }
        }
        // read a database directory to list a git crate in form of crate_name-HEAD
        if db_dir.exists() && db_dir.is_dir() {
            for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                let entry_path = entry?.path();
                let crate_size =
                    get_size(&entry_path).context("failed to get size of db dir folders")?;
                let file_name = entry_path.file_name().context("failed to get file name")?;
                let file_name_str = file_name
                    .to_str()
                    .context("failed to convert db dir file name to str")?;
                let splitted_file_name = file_name_str.rsplitn(2, '-').collect::<Vec<_>>();
                let crate_name_initial = splitted_file_name[1];
                let full_name = format!("{crate_name_initial}-HEAD");
                let crate_metadata = CrateMetaData {
                    name: full_name,
                    version: None,
                    size: crate_size,
                    source: entry_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .map(ToString::to_string),
                    path: Some(entry_path),
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
