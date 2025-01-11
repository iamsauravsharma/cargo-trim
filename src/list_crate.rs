use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use url::Url;

use crate::config_file::ConfigFile;
use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::dir_path::DirPath;

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

/// struct to store all crate list detail with its type
pub(crate) struct CrateList {
    installed_bin: Vec<CrateMetaData>,
    installed_crate_registry: Vec<CrateMetaData>,
    installed_crate_git: Vec<CrateMetaData>,
    old_crate_registry: Vec<CrateMetaData>,
    old_crate_git: Vec<CrateMetaData>,
    orphan_crate_registry: Vec<CrateMetaData>,
    orphan_crate_git: Vec<CrateMetaData>,
    cargo_lock_files: CargoLockFiles,
}

impl CrateList {
    /// create list of all types of crate present in directory
    pub(crate) fn create_list(
        dir_path: &DirPath,
        config_file: &ConfigFile,
        crate_detail: &mut CrateDetail,
    ) -> Result<Self> {
        let bin_dir = dir_path.bin_dir();
        let cache_dir = dir_path.cache_dir();
        let src_dir = dir_path.src_dir();
        let checkout_dir = dir_path.checkout_dir();
        let db_dir = dir_path.db_dir();

        // list installed crates
        let installed_bin = crate_detail.list_installed_bin(bin_dir)?;
        let installed_crate_registry =
            crate_detail.list_installed_crate_registry(src_dir, cache_dir)?;
        let installed_crate_git = crate_detail.list_installed_crate_git(checkout_dir, db_dir)?;

        // list old registry crate
        let (old_crate_registry, old_crate_git) = list_old_crates(
            db_dir,
            installed_crate_registry.clone(),
            &installed_crate_git,
        )?;

        // list all used crates in rust program
        let (cargo_lock_files, used_crate_registry, used_crate_git) =
            list_used_crates(config_file, crate_detail)?;

        // list orphan crates. If crate is not used then it is orphan
        let (orphan_crate_registry, orphan_crate_git) = list_orphan_crates(
            &installed_crate_registry,
            &installed_crate_git,
            &used_crate_registry,
            &used_crate_git,
        );

        Ok(Self {
            installed_bin,
            installed_crate_registry,
            installed_crate_git,
            old_crate_registry,
            old_crate_git,
            orphan_crate_registry,
            orphan_crate_git,
            cargo_lock_files,
        })
    }

    /// provide list of installed bin
    pub(crate) fn installed_bin(&self) -> &Vec<CrateMetaData> {
        &self.installed_bin
    }

    /// provide list of installed registry
    pub(crate) fn installed_registry(&self) -> &Vec<CrateMetaData> {
        &self.installed_crate_registry
    }

    /// provide list of old registry
    pub(crate) fn old_registry(&self) -> &Vec<CrateMetaData> {
        &self.old_crate_registry
    }

    /// provide list o orphan registry
    pub(crate) fn orphan_registry(&self) -> &Vec<CrateMetaData> {
        &self.orphan_crate_registry
    }

    /// provide list of installed git
    pub(crate) fn installed_git(&self) -> &Vec<CrateMetaData> {
        &self.installed_crate_git
    }

    /// provide list of old git
    pub(crate) fn old_git(&self) -> &Vec<CrateMetaData> {
        &self.old_crate_git
    }

    /// provide list of orphan git
    pub(crate) fn orphan_git(&self) -> &Vec<CrateMetaData> {
        &self.orphan_crate_git
    }

    /// List Cargo.lock file
    pub(crate) fn cargo_lock_files(&self) -> &CargoLockFiles {
        &self.cargo_lock_files
    }

    /// list crates which is both old and orphan
    pub(crate) fn old_orphan_registry(&self) -> Vec<CrateMetaData> {
        let mut old_orphan_registry = Vec::new();
        let orphan_list = self.orphan_registry();
        for crates in self.old_registry() {
            if orphan_list.contains(crates) {
                old_orphan_registry.push(crates.clone());
            }
        }
        old_orphan_registry
    }

    /// List git crates which is both old and orphan
    pub(crate) fn old_orphan_git(&self) -> Vec<CrateMetaData> {
        let mut old_orphan_git = Vec::new();
        let orphan_list = self.orphan_git();
        for crates in self.old_git() {
            if orphan_list.contains(crates) {
                old_orphan_git.push(crates.clone());
            }
        }
        old_orphan_git
    }
}

/// Read out content of Cargo.lock file to List crates present so can be
/// used for orphan clean
fn read_content(
    cargo_lock_paths: &[PathBuf],
    crate_detail: &CrateDetail,
) -> Result<(Vec<CrateMetaData>, Vec<CrateMetaData>)> {
    let mut present_crate_registry = Vec::new();
    let mut present_crate_git = Vec::new();
    for cargo_lock_file in cargo_lock_paths {
        if cargo_lock_file.exists() {
            let file_content = std::fs::read_to_string(cargo_lock_file)
                .context("failed to read cargo lock content to string")?;
            let cargo_lock_data: LockData =
                toml::from_str(&file_content).context("failed to convert to toml format")?;
            if let Some(packages) = cargo_lock_data.package() {
                for package in packages {
                    if let Some(source) = package.source() {
                        let name = package.name();
                        let version = package.version();
                        if source.contains("registry+") {
                            let mut url = Url::from_str(&source.replace("registry+", ""))
                                .context("failed registry source url kind conversion")?;
                            // Only add sparse registry if sparse registry is present in place of
                            // git based registry for crates.io
                            let index_crates_url = Url::from_str("https://index.crates.io")?;
                            if url == Url::from_str("https://github.com/rust-lang/crates.io-index")?
                                && crate_detail
                                    .source_infos()
                                    .values()
                                    .collect::<Vec<_>>()
                                    .contains(&&index_crates_url)
                            {
                                url = index_crates_url;
                            }
                            for index_name in crate_detail.index_names_from_url(&url) {
                                present_crate_registry.push(CrateMetaData::new(
                                    name.to_string(),
                                    Some(Version::parse(version).context(
                                        "failed Cargo.lock semver
                            version parse",
                                    )?),
                                    Some(index_name),
                                ));
                            }
                        }
                        if source.contains("git+") {
                            let url_with_kind;
                            let rev_sha_vec: Vec<&str>;
                            // determine url with kind according to git source have query param or
                            // not
                            if source.contains("?rev=")
                                || source.contains("?branch=")
                                || source.contains("?tag=")
                            {
                                let split_url: Vec<&str> = if source.contains("?rev=") {
                                    source.split("?rev=").collect()
                                } else if source.contains("?branch=") {
                                    source.split("?branch=").collect()
                                } else {
                                    source.split("?tag=").collect()
                                };
                                rev_sha_vec = split_url[1].split('#').collect();
                                url_with_kind = split_url[0];
                            } else {
                                rev_sha_vec = source.split('#').collect();
                                url_with_kind = rev_sha_vec[0];
                            }
                            let rev_short_form = &rev_sha_vec[1][..=6];
                            let url = Url::from_str(&url_with_kind.replace("git+", "")).context(
                                "failed git source url kind with query params conversion",
                            )?;
                            let last_path_segment = url
                                .path_segments()
                                .context("url doesn't have segment")?
                                .last()
                                .context("cannot get last segments of path")?;
                            let full_name = format!("{last_path_segment}-{rev_short_form}");
                            for index_name in crate_detail.index_names_from_url(&url) {
                                present_crate_git.push(CrateMetaData::new(
                                    full_name.clone(),
                                    None,
                                    Some(index_name),
                                ));
                            }
                        }
                        if source.contains("sparse+") {
                            let url = Url::from_str(&source.replace("sparse+", ""))
                                .context("failed sparse source url kind conversion")?;
                            for index_name in crate_detail.index_names_from_url(&url) {
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

/// List old crates
fn list_old_crates(
    db_dir: &Path,
    installed_crate_registry: Vec<CrateMetaData>,
    installed_crate_git: &[CrateMetaData],
) -> Result<(Vec<CrateMetaData>, Vec<CrateMetaData>)> {
    let mut old_crate_registry = Vec::new();
    let mut registry_crates = installed_crate_registry;
    registry_crates.sort();
    if registry_crates.len() > 1 {
        for i in 0..(registry_crates.len() - 1) {
            let crate_metadata = &registry_crates[i];
            let next_crate_metadata = &registry_crates[i + 1];
            if crate_metadata.name() == next_crate_metadata.name()
                && crate_metadata.source() == next_crate_metadata.source()
            {
                old_crate_registry.push(crate_metadata.clone());
            }
        }
    }
    old_crate_registry.sort();
    old_crate_registry.dedup();

    // list old git crate
    let mut old_crate_git = Vec::new();
    // analyze each crates of db dir and create list of head rev value
    if db_dir.exists() && db_dir.is_dir() {
        let mut full_name_list = Vec::new();
        for crates in fs::read_dir(db_dir).context("failed to read db dir")? {
            let entry = crates?.path();
            let file_name = entry
                .file_name()
                .context("failed to get sold crate db dir file name")?
                .to_str()
                .context("failed to convert db dir entry file name to str")?;
            let rev_value = latest_rev_value(&entry)?;
            let full_name = format!("{file_name}-{rev_value}");
            full_name_list.push(full_name);
        }
        for crate_metadata in installed_crate_git {
            let crate_name = crate_metadata.name();
            if !crate_name.contains("-HEAD") && !full_name_list.contains(crate_name) {
                old_crate_git.push(crate_metadata.clone());
            }
        }
    }
    old_crate_git.sort();
    old_crate_git.dedup();

    Ok((old_crate_registry, old_crate_git))
}

/// list used crates
fn list_used_crates(
    config_file: &ConfigFile,
    crate_detail: &CrateDetail,
) -> Result<(CargoLockFiles, Vec<CrateMetaData>, Vec<CrateMetaData>)> {
    let mut used_crate_registry = Vec::new();
    let mut used_crate_git = Vec::new();
    let mut cargo_lock_files = CargoLockFiles::new();
    let config_directory = config_file.directory().clone();
    // read a Cargo.lock file and determine out a used registry and git crate
    for path in &config_directory {
        let list_cargo_locks = config_file.list_cargo_locks(Path::new(path))?;
        let (mut registry_crate, mut git_crate) =
            read_content(list_cargo_locks.paths(), crate_detail)?;
        cargo_lock_files.append(list_cargo_locks);
        used_crate_registry.append(&mut registry_crate);
        used_crate_git.append(&mut git_crate);
    }
    used_crate_registry.sort();
    used_crate_registry.dedup();
    used_crate_git.sort();
    used_crate_registry.dedup();
    Ok((cargo_lock_files, used_crate_registry, used_crate_git))
}

/// list orphan crates
fn list_orphan_crates(
    installed_crate_registry: &[CrateMetaData],
    installed_crate_git: &[CrateMetaData],
    used_crate_registry: &[CrateMetaData],
    used_crate_git: &[CrateMetaData],
) -> (Vec<CrateMetaData>, Vec<CrateMetaData>) {
    let mut orphan_crate_registry = Vec::new();
    let mut orphan_crate_git = Vec::new();
    for crates in installed_crate_registry {
        if !used_crate_registry.contains(crates) {
            orphan_crate_registry.push(crates.clone());
        }
    }
    for installed_crate_metadata in installed_crate_git {
        let crate_name = installed_crate_metadata.name();
        if crate_name.contains("-HEAD") {
            if used_crate_git.is_empty() {
                orphan_crate_git.push(installed_crate_metadata.clone());
            }
            if !used_crate_git
                .iter()
                .any(|used| used.source() == installed_crate_metadata.source())
            {
                orphan_crate_git.push(installed_crate_metadata.clone());
            }
        } else if !used_crate_git.contains(installed_crate_metadata) {
            orphan_crate_git.push(installed_crate_metadata.clone());
        }
    }
    orphan_crate_registry.sort();
    orphan_crate_registry.dedup();
    orphan_crate_git.sort();
    orphan_crate_git.dedup();
    (orphan_crate_registry, orphan_crate_git)
}

/// get latest commit rev value from git repository
fn latest_rev_value(path: &Path) -> Result<String> {
    let mut fetch_head_file = PathBuf::new();
    fetch_head_file.push(path);
    fetch_head_file.push("FETCH_HEAD");
    let content = fs::read_to_string(fetch_head_file).context("failed to read FETCH_HEAD file")?;
    // read first 7 value which is same as hash for git based checkout folder
    Ok(content[..7].to_string())
}
