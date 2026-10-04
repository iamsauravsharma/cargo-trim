use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;

use anyhow::{Context as _, Result};
use semver::Version;
use serde::Deserialize;
use url::Url;

use crate::config_file::ConfigFile;
use crate::installed::{CrateMetaData, Sources};

/// struct to store Cargo.lock location
pub(crate) struct CargoLockFiles {
    path: Vec<PathBuf>,
}

impl CargoLockFiles {
    pub(crate) fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub(crate) fn add_path(&mut self, path: PathBuf) {
        self.path.push(path);
    }

    pub(crate) fn append(&mut self, mut lock_location: Self) {
        self.path.append(&mut lock_location.path);
    }

    pub(crate) fn paths(&self) -> &Vec<PathBuf> {
        &self.path
    }
}

#[derive(Clone, Deserialize)]
struct LockData {
    package: Option<Vec<Package>>,
}

impl LockData {
    fn package(&self) -> Option<&Vec<Package>> {
        self.package.as_ref()
    }
}

#[derive(Clone, Deserialize)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
}

impl Package {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }
}

/// Parse a `git+…` source string from Cargo.lock and return `(repo_url,
/// short_sha)`.
fn parse_git_source(source: &str) -> Result<(Url, String)> {
    let base_and_query_optional = source
        .split_once("?rev=")
        .or_else(|| source.split_once("?branch="))
        .or_else(|| source.split_once("?tag="));
    let (url_with_kind, sha_part) = if let Some((base, query_and_hash)) = base_and_query_optional {
        let sha_part = query_and_hash
            .split_once('#')
            .context("failed to find # in git source query param")?
            .1;
        (base, sha_part)
    } else {
        source
            .split_once('#')
            .context("failed to find # in git source")?
    };
    let rev_short_form = sha_part
        .get(..7)
        .context("git SHA in Cargo.lock is shorter than 7 characters")?
        .to_string();
    let url = Url::from_str(url_with_kind.strip_prefix("git+").unwrap_or(url_with_kind))
        .context("failed git source url kind with query params conversion")?;
    Ok((url, rev_short_form))
}

/// Read out content of Cargo.lock file to List crates present so can be
/// used for orphan clean
fn read_content(
    cargo_lock_paths: &[PathBuf],
    sources: &Sources,
) -> Result<(Vec<CrateMetaData>, Vec<CrateMetaData>)> {
    let mut present_crate_registry = Vec::new();
    let mut present_crate_git = Vec::new();
    let crates_io_git_url = Url::from_str("https://github.com/rust-lang/crates.io-index")?;
    let index_crates_url = Url::from_str("https://index.crates.io")?;
    let sparse_crates_io_present = sources.has_url(&index_crates_url);
    for cargo_lock_file in cargo_lock_paths {
        if cargo_lock_file.exists() {
            let file_content = fs::read_to_string(cargo_lock_file)
                .context("failed to read cargo lock content to string")?;
            let cargo_lock_data: LockData =
                toml::from_str(&file_content).context("failed to convert to toml format")?;
            if let Some(packages) = cargo_lock_data.package() {
                for package in packages {
                    if let Some(source) = package.source() {
                        let name = package.name();
                        let version = package.version();
                        if let Some(registry_url) = source.strip_prefix("registry+") {
                            let mut url = Url::from_str(registry_url)
                                .context("failed registry source url kind conversion")?;
                            // Only add sparse registry if sparse registry is
                            // present in place of
                            // git based registry for crates.io
                            if url == crates_io_git_url && sparse_crates_io_present {
                                url = index_crates_url.clone();
                            }
                            for index_name in sources.names_for_url(&url) {
                                present_crate_registry.push(CrateMetaData::new(
                                    name.to_string(),
                                    Some(
                                        Version::parse(version)
                                            .context("failed Cargo.lock semver version parse")?,
                                    ),
                                    Some(index_name),
                                ));
                            }
                        } else if source.starts_with("git+") {
                            let (url, rev_short_form) = parse_git_source(source)?;
                            let last_path_segment = url
                                .path_segments()
                                .context("url doesn't have segment")?
                                .next_back()
                                .context("cannot get last segments of path")?;
                            let full_name = format!("{last_path_segment}-{rev_short_form}");
                            for index_name in sources.names_for_url(&url) {
                                present_crate_git.push(CrateMetaData::new(
                                    full_name.clone(),
                                    None,
                                    Some(index_name),
                                ));
                            }
                        } else if let Some(sparse_url) = source.strip_prefix("sparse+") {
                            let url = Url::from_str(sparse_url)
                                .context("failed sparse source url kind conversion")?;
                            for index_name in sources.names_for_url(&url) {
                                present_crate_registry.push(CrateMetaData::new(
                                    name.to_string(),
                                    Some(
                                        Version::parse(version)
                                            .context("failed Cargo.lock semver version parse")?,
                                    ),
                                    Some(index_name),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok((present_crate_registry, present_crate_git))
}

/// list Cargo.lock files of every project directory along with the registry
/// and git crates they use, both lists sorted
pub(crate) fn used_crates(
    config_file: &ConfigFile,
    sources: &Sources,
) -> Result<(CargoLockFiles, Vec<CrateMetaData>, Vec<CrateMetaData>)> {
    let mut used_crate_registry = Vec::new();
    let mut used_crate_git = Vec::new();
    let mut cargo_lock_files = CargoLockFiles::new();
    let config_directory = config_file.directory().clone();
    // read a Cargo.lock file and determine out a used registry and git crate
    for path in &config_directory {
        let list_cargo_locks = config_file.list_cargo_locks(Path::new(path))?;
        let (mut registry_crate, mut git_crate) = read_content(list_cargo_locks.paths(), sources)?;
        cargo_lock_files.append(list_cargo_locks);
        used_crate_registry.append(&mut registry_crate);
        used_crate_git.append(&mut git_crate);
    }
    used_crate_registry.sort();
    used_crate_registry.dedup();
    used_crate_git.sort();
    used_crate_git.dedup();
    Ok((cargo_lock_files, used_crate_registry, used_crate_git))
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::parse_git_source;

    #[test]
    fn parse_git_source_plain_hash_test() {
        let (url, sha) =
            parse_git_source("git+https://github.com/foo/bar#0123456789abcdef0123456789ab")
                .unwrap();
        assert_eq!(url, Url::parse("https://github.com/foo/bar").unwrap());
        assert_eq!(sha, "0123456");
    }

    #[test]
    fn parse_git_source_rev_query_test() {
        let (url, sha) =
            parse_git_source("git+https://github.com/foo/bar?rev=v1.2.3#abcdef1234567890").unwrap();
        assert_eq!(url, Url::parse("https://github.com/foo/bar").unwrap());
        assert_eq!(sha, "abcdef1");
    }

    #[test]
    fn parse_git_source_branch_query_test() {
        let (url, sha) =
            parse_git_source("git+https://github.com/foo/bar?branch=main#deadbeefcafe0").unwrap();
        assert_eq!(url, Url::parse("https://github.com/foo/bar").unwrap());
        assert_eq!(sha, "deadbee");
    }

    #[test]
    fn parse_git_source_tag_query_test() {
        let (url, sha) =
            parse_git_source("git+https://github.com/foo/bar?tag=v0.1.0#0f1e2d3c4b5a").unwrap();
        assert_eq!(url, Url::parse("https://github.com/foo/bar").unwrap());
        assert_eq!(sha, "0f1e2d3");
    }

    #[test]
    fn parse_git_source_missing_hash_is_error_test() {
        assert!(parse_git_source("git+https://github.com/foo/bar").is_err());
    }

    #[test]
    fn parse_git_source_short_sha_is_error_test() {
        assert!(parse_git_source("git+https://github.com/foo/bar#abc").is_err());
    }

    #[test]
    fn parse_git_source_rev_without_hash_is_error_test() {
        assert!(parse_git_source("git+https://github.com/foo/bar?rev=v1").is_err());
    }
}
