use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;
use std::{fmt, fs, io};

use anyhow::{Context as _, Result};
use semver::Version;
use serde::Deserialize;
use url::Url;

use crate::utils::{get_size, split_name_version};

/// installed crate. A crate present in several cache folders, such as registry
/// source and archive, is a single entry holding every path and their total
/// size
#[derive(Debug, Clone)]
pub(crate) struct CrateMetaData {
    name: String,
    version: Option<Version>,
    size: u64,
    source: Option<String>,
    paths: Vec<PathBuf>,
}

impl CrateMetaData {
    pub(crate) fn new(name: String, version: Option<Version>, source: Option<String>) -> Self {
        Self {
            name,
            version,
            size: 0,
            source,
            paths: Vec::new(),
        }
    }

    /// crate found at path, with size of path
    fn located(
        name: String,
        version: Option<Version>,
        source: Option<String>,
        path: PathBuf,
    ) -> Result<Self> {
        let size =
            get_size(&path).with_context(|| format!("failed to get size of {}", path.display()))?;
        Ok(Self {
            name,
            version,
            size,
            source,
            paths: vec![path],
        })
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

    /// every path holding the crate
    pub(crate) fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// `source name-version`, leaving out parts which are not known
impl fmt::Display for CrateMetaData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = &self.source {
            write!(f, "{source} ")?;
        }
        write!(f, "{}", self.name)?;
        if let Some(version) = &self.version {
            write!(f, "-{version}")?;
        }
        Ok(())
    }
}

impl PartialOrd for CrateMetaData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CrateMetaData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.version.cmp(&other.version))
            .then_with(|| self.source.cmp(&other.source))
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

/// remote url of every registry index and git db folder, keyed by folder name
pub(crate) struct Sources {
    urls: HashMap<String, Url>,
}

impl Sources {
    pub(crate) fn new(index_dir: &Path, db_dir: &Path) -> Result<Self> {
        let mut urls = HashMap::new();
        for registry_dir in sub_dirs(index_dir)? {
            let name = dir_name(&registry_dir)?;
            // a git based index records its remote in FETCH_HEAD while a sparse
            // index only has config.json and its folder name holds the domain
            let fetch_head_file = registry_dir.join(".git").join("FETCH_HEAD");
            let config_file = registry_dir.join("config.json");
            if fetch_head_file.exists() {
                if let Some(url) = fetch_head_url(&fetch_head_file)? {
                    urls.insert(name, url);
                }
            } else if config_file.exists() {
                let domain = name
                    .rsplit_once('-')
                    .map_or(name.as_str(), |(domain, _)| domain);
                let content =
                    fs::read_to_string(&config_file).context("failed to read config.json file")?;
                let json: IndexConfig = serde_json::from_str(&content)?;
                // folder name has no scheme so take it from api url, or dl url
                // when api url is missing
                let scheme_url = json.api.unwrap_or(json.dl);
                let url = Url::from_str(&format!("{}://{domain}", scheme_url.scheme()))
                    .context("failed sparse registry index url")?;
                urls.insert(name, url);
            }
        }
        for git_dir in sub_dirs(db_dir)? {
            if let Some(url) = fetch_head_url(&git_dir.join("FETCH_HEAD"))? {
                urls.insert(dir_name(&git_dir)?, url);
            }
        }
        Ok(Self { urls })
    }

    /// folder names whose remote is url
    pub(crate) fn names_for_url(&self, url: &Url) -> Vec<String> {
        self.urls
            .iter()
            .filter(|(_, value)| *value == url)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// check if any folder has url as remote
    pub(crate) fn has_url(&self, url: &Url) -> bool {
        self.urls.values().any(|value| value == url)
    }

    /// every folder name
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.urls.keys().map(String::as_str)
    }
}

/// list installed binaries
pub(crate) fn installed_bin(bin_dir: &Path) -> Result<Vec<CrateMetaData>> {
    let mut installed = Vec::new();
    if bin_dir.is_dir() {
        for entry in fs::read_dir(bin_dir).context("failed to read bin directory")? {
            let path = entry?.path();
            installed.push(CrateMetaData::located(dir_name(&path)?, None, None, path)?);
        }
    }
    Ok(merge(installed))
}

/// list installed registry crates of both source and archive folder
pub(crate) fn installed_registry(src_dir: &Path, cache_dir: &Path) -> Result<Vec<CrateMetaData>> {
    let mut installed = Vec::new();
    for base_dir in [src_dir, cache_dir] {
        for registry in sub_dirs(base_dir)? {
            let source = dir_name(&registry)?;
            for entry in fs::read_dir(&registry).context("failed to read registry folder")? {
                let path = entry?.path();
                let (name, version) = split_name_version(&dir_name(&path)?)?;
                installed.push(CrateMetaData::located(
                    name,
                    Some(version),
                    Some(source.clone()),
                    path,
                )?);
            }
        }
    }
    Ok(merge(installed))
}

/// list installed git crates. A checkout is named `repo-rev` while a git db is
/// named `repo-HEAD`
pub(crate) fn installed_git(checkout_dir: &Path, db_dir: &Path) -> Result<Vec<CrateMetaData>> {
    let mut installed = Vec::new();
    for repo in sub_dirs(checkout_dir)? {
        let source = dir_name(&repo)?;
        let repo_name = repo_name(&source)?;
        for entry in fs::read_dir(&repo).context("failed to read checkout folder")? {
            let path = entry?.path();
            let name = format!("{repo_name}-{}", dir_name(&path)?);
            installed.push(CrateMetaData::located(
                name,
                None,
                Some(source.clone()),
                path,
            )?);
        }
    }
    for path in sub_dirs(db_dir)? {
        let source = dir_name(&path)?;
        let name = format!("{}-HEAD", repo_name(&source)?);
        installed.push(CrateMetaData::located(name, None, Some(source), path)?);
    }
    Ok(merge(installed))
}

/// sort crates and merge equal ones into a single entry
fn merge(mut crates: Vec<CrateMetaData>) -> Vec<CrateMetaData> {
    crates.sort();
    let mut merged: Vec<CrateMetaData> = Vec::with_capacity(crates.len());
    for crate_metadata in crates {
        match merged.last_mut() {
            Some(last) if *last == crate_metadata => {
                last.size += crate_metadata.size;
                last.paths.extend(crate_metadata.paths);
            }
            _ => merged.push(crate_metadata),
        }
    }
    merged
}

/// folders directly inside dir, none when dir does not exist
fn sub_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

/// last component of path as string
fn dir_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToString::to_string)
        .with_context(|| format!("failed to get file name of {}", path.display()))
}

/// repository name of a git folder named `repo-hash`
fn repo_name(folder_name: &str) -> Result<&str> {
    folder_name
        .rsplit_once('-')
        .map(|(name, _)| name)
        .with_context(|| format!("failed to split git folder name {folder_name}"))
}

/// read the remote url a git repository recorded in its `FETCH_HEAD` file,
/// `None` if the repository has no usable `FETCH_HEAD`
fn fetch_head_url(fetch_head_file: &Path) -> Result<Option<Url>> {
    let content = match fs::read_to_string(fetch_head_file) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read {}", fetch_head_file.display()));
        }
    };
    let Some(url_path) = content.split_whitespace().last() else {
        return Ok(None);
    };
    Url::from_str(url_path)
        .with_context(|| format!("failed to convert url of {}", fetch_head_file.display()))
        .map(Some)
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::path::PathBuf;

    use semver::Version;

    use super::{CrateMetaData, merge};

    fn meta(name: &str, version: &str, source: &str, size: u64) -> CrateMetaData {
        CrateMetaData {
            name: name.to_string(),
            version: Some(Version::parse(version).unwrap()),
            size,
            source: Some(source.to_string()),
            paths: vec![PathBuf::from(format!("/{source}/{name}-{version}-{size}"))],
        }
    }

    #[test]
    fn crate_metadata_equality_ignores_size_test() {
        let a = meta("serde", "1.0.0", "registry", 100);
        let b = meta("serde", "1.0.0", "registry", 999);
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn crate_metadata_ordering_prefers_name_then_version_then_source_test() {
        let base = meta("serde", "1.0.0", "registry", 0);
        // name has the highest priority
        assert!(meta("aaa", "9.9.9", "zzz", 0) < base);
        // then version
        assert!(base < meta("serde", "1.2.0", "registry", 0));
        // then source
        assert!(meta("serde", "1.0.0", "aaa", 0) < base);
    }

    #[test]
    fn merge_combines_size_and_paths_of_equal_entry_test() {
        let merged = merge(vec![
            meta("serde", "1.0.0", "registry", 100),
            meta("tokio", "1.0.0", "registry", 10),
            meta("serde", "1.0.0", "registry", 50),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].size(), 150);
        assert_eq!(merged[0].paths().len(), 2);
        assert_eq!(merged[1].paths().len(), 1);
    }

    #[test]
    fn merge_keeps_distinct_entries_test() {
        let merged = merge(vec![
            meta("serde", "2.0.0", "registry", 100),
            meta("serde", "1.0.0", "registry", 100),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].version(), Some(&Version::parse("1.0.0").unwrap()));
    }

    #[test]
    fn display_test() {
        assert_eq!(
            meta("serde", "1.0.0", "idx", 0).to_string(),
            "idx serde-1.0.0"
        );
        assert_eq!(
            CrateMetaData::new("repo-abc".to_string(), None, Some("repo-h".to_string()))
                .to_string(),
            "repo-h repo-abc"
        );
        assert_eq!(
            CrateMetaData::new("cargo-trim".to_string(), None, None).to_string(),
            "cargo-trim"
        );
    }
}
