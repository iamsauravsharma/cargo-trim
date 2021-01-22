use crate::{
    config_file::ConfigFile, crate_detail::CrateDetail, dir_path::DirPath,
    utils::clear_version_value,
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

// struct store Cargo.toml file location
pub(crate) struct CargoTomlLocation {
    path: Vec<PathBuf>,
}

impl CargoTomlLocation {
    pub(crate) fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub(crate) fn add_path(&mut self, path: PathBuf) {
        self.path.push(path);
    }

    pub(crate) fn append(&mut self, mut lock_location: Self) {
        self.path.append(&mut lock_location.path);
    }

    pub(crate) fn location_path(&self) -> &Vec<PathBuf> {
        &self.path
    }
}

#[derive(Clone, Deserialize)]
struct LockData {
    package: Option<Vec<Package>>,
}

impl LockData {
    fn package(&self) -> &Option<Vec<Package>> {
        &self.package
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

    fn source(&self) -> &Option<String> {
        &self.source
    }
}

// struct to store all crate list detail with its type
pub(crate) struct CrateList {
    installed_bin: Vec<String>,
    installed_crate_registry: Vec<String>,
    installed_crate_git: Vec<String>,
    old_crate_registry: Vec<String>,
    old_crate_git: Vec<String>,
    used_crate_registry: Vec<String>,
    used_crate_git: Vec<String>,
    orphan_crate_registry: Vec<String>,
    orphan_crate_git: Vec<String>,
    cargo_toml_location: CargoTomlLocation,
}

impl CrateList {
    // create list of all types of crate present in directory
    #[allow(clippy::too_many_lines)]
    pub(crate) fn create_list(
        dir_path: &DirPath,
        config_file: &ConfigFile,
        crate_detail: &mut CrateDetail,
    ) -> Self {
        let bin_dir = dir_path.bin_dir().as_path();
        let cache_dir = dir_path.cache_dir();
        let src_dir = dir_path.src_dir();
        let checkout_dir = dir_path.checkout_dir().as_path();
        let db_dir = dir_path.db_dir().as_path();

        // list installed crates
        let installed_bin = crate_detail.get_installed_bin(bin_dir);
        let installed_crate_registry =
            crate_detail.get_installed_crate_registry(src_dir, cache_dir);
        let installed_crate_git = crate_detail.get_installed_crate_git(checkout_dir, db_dir);

        // list old registry crate
        let mut old_crate_registry = Vec::new();
        let mut version_removed_crate = remove_version(&installed_crate_registry);
        version_removed_crate.sort();
        if !version_removed_crate.is_empty() {
            let mut common_crate_version = Vec::new();
            // check a two version of same crate if crate has two version then it is added
            // to vec and checked to determine old crate version
            for i in 0..(version_removed_crate.len() - 1) {
                let (name, version) = &version_removed_crate[i];
                let (next_name, _) = &version_removed_crate[i + 1];
                if name == next_name {
                    common_crate_version.push((version, i));
                } else {
                    common_crate_version.push((version, i));
                    let mut latest_version = version;
                    // this is used to check which version is latest since we have sorted string so
                    // 0.10.0 will come before 0.9.0. so it need to be done to properly determine
                    // latest version
                    for (common_version, _) in &common_crate_version {
                        if semver::Version::parse(latest_version)
                            < semver::Version::parse(common_version)
                        {
                            latest_version = common_version;
                        }
                    }
                    for (crate_version, position) in &common_crate_version {
                        if crate_version.as_str() != latest_version {
                            old_crate_registry
                                .push(installed_crate_registry.get(*position).unwrap().to_string());
                        }
                    }
                    common_crate_version = Vec::new()
                }
            }
        }
        old_crate_registry.sort();
        old_crate_registry.dedup();

        // list old git crate
        let mut old_crate_git = Vec::new();
        let mut full_name_list = Vec::new();
        // analyze each crates of db dir and create list of head rev value
        for crates in fs::read_dir(db_dir).expect("failed to read db dir") {
            let entry = crates.unwrap().path();
            let path = entry.as_path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
            let mut rev_value = latest_rev_value(path);
            rev_value.retain(|c| c != '\'');
            let full_name = format!("{}-{}", name[1], rev_value);
            full_name_list.push(full_name)
        }
        for crate_name in &installed_crate_git {
            if !crate_name.contains("-HEAD") {
                // check if crate is in rev value else list that crate as old crate
                if !full_name_list.contains(crate_name) {
                    old_crate_git.push(crate_name.to_string());
                }
            }
        }
        old_crate_git.sort();
        old_crate_git.dedup();

        // list all used crates in rust program
        let mut used_crate_registry = Vec::new();
        let mut used_crate_git = Vec::new();
        let mut cargo_toml_location = CargoTomlLocation::new();
        let config_directory = config_file.directory().to_owned();
        // read a Cargo.lock file and determine out a used registry and git crate
        for path in &config_directory {
            let list_cargo_toml = config_file.list_cargo_toml(&Path::new(path));
            let (mut registry_crate, mut git_crate) = read_content(list_cargo_toml.location_path());
            cargo_toml_location.append(list_cargo_toml);
            used_crate_registry.append(&mut registry_crate);
            used_crate_git.append(&mut git_crate);
        }
        used_crate_registry.sort();
        used_crate_registry.dedup();
        used_crate_git.sort();
        used_crate_registry.dedup();

        // list orphan crates. If crate is not used then it is orphan
        let mut orphan_crate_registry = Vec::new();
        let mut orphan_crate_git = Vec::new();
        for crates in &installed_crate_registry {
            if !used_crate_registry.contains(crates) {
                orphan_crate_registry.push(crates.to_string());
            }
        }
        for crates in &installed_crate_git {
            if crates.contains("-HEAD") {
                let split_installed = crates.rsplitn(2, '-').collect::<Vec<&str>>();
                if used_crate_git.is_empty() {
                    orphan_crate_git.push(crates.to_string());
                }
                let mut used_in_project = false;
                for used in &used_crate_git {
                    if used.contains(split_installed[1]) {
                        used_in_project = true;
                        // Break if found to be used one time no need to check for other
                        break;
                    }
                }
                if !used_in_project {
                    orphan_crate_git.push(crates.to_string());
                }
            } else if !used_crate_git.contains(crates) {
                orphan_crate_git.push(crates.to_string());
            }
        }
        orphan_crate_registry.sort();
        orphan_crate_registry.dedup();
        orphan_crate_git.sort();
        orphan_crate_git.dedup();

        Self {
            installed_bin,
            installed_crate_registry,
            installed_crate_git,
            old_crate_registry,
            old_crate_git,
            used_crate_registry,
            used_crate_git,
            orphan_crate_registry,
            orphan_crate_git,
            cargo_toml_location,
        }
    }

    // provide list of installed bin
    pub(crate) fn installed_bin(&self) -> &Vec<String> {
        &self.installed_bin
    }

    // provide list of installed registry
    pub(crate) fn installed_registry(&self) -> &Vec<String> {
        &self.installed_crate_registry
    }

    // provide list of old registry
    pub(crate) fn old_registry(&self) -> &Vec<String> {
        &self.old_crate_registry
    }

    // provide list of used registry
    pub(crate) fn used_registry(&self) -> &Vec<String> {
        &self.used_crate_registry
    }

    // provide list o orphan registry
    pub(crate) fn orphan_registry(&self) -> &Vec<String> {
        &self.orphan_crate_registry
    }

    // provide list of installed git
    pub(crate) fn installed_git(&self) -> &Vec<String> {
        &self.installed_crate_git
    }

    // provide list of old git
    pub(crate) fn old_git(&self) -> &Vec<String> {
        &self.old_crate_git
    }

    // provide list of used git
    pub(crate) fn used_git(&self) -> &Vec<String> {
        &self.used_crate_git
    }

    // provide list of orphan git
    pub(crate) fn orphan_git(&self) -> &Vec<String> {
        &self.orphan_crate_git
    }

    // list out path of directory which contains cargo lock file
    pub(crate) fn cargo_toml_location(&self) -> &CargoTomlLocation {
        &self.cargo_toml_location
    }
}

// Read out content of cargo.lock file to list out crates present so can be used
// for orphan clean
fn read_content(list: &[PathBuf]) -> (Vec<String>, Vec<String>) {
    let mut present_crate_registry = Vec::new();
    let mut present_crate_git = Vec::new();
    for lock in list.iter() {
        let mut lock_folder = lock.clone();
        lock_folder.push("Cargo.lock");
        if lock_folder.exists() {
            let lock_file = lock_folder
                .to_str()
                .expect("Failed to convert lock_folder to str");
            let file_content = std::fs::read_to_string(lock_file)
                .expect("failed to read cargo lock content to string");
            let cargo_lock_data: LockData =
                toml::from_str(&file_content).expect("Failed to convert to Toml format");
            if let Some(packages) = cargo_lock_data.package() {
                for package in packages {
                    if let Some(source) = package.source() {
                        let name = package.name();
                        let version = package.version();
                        if source.contains("registry+") {
                            let full_name = format!("{}-{}", name, version);
                            present_crate_registry.push(full_name);
                        }
                        if source.contains("git+") {
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
                                let rev_sha: Vec<&str> = split_url[1].split('#').collect();
                                let rev_value = rev_sha[1];
                                let rev_short_form = &rev_value[..=6];
                                let full_name = format!("{}-{}", name, rev_short_form);
                                present_crate_git.push(full_name);
                            } else {
                                let rev_sha: Vec<&str> = source.split('#').collect();
                                let rev_value = rev_sha[1];
                                let rev_short_form = &rev_value[..=6];
                                let full_name = format!("{}-{}", name, rev_short_form);
                                present_crate_git.push(full_name);
                            }
                        }
                    }
                }
            }
        }
    }
    (present_crate_registry, present_crate_git)
}

// Function used to remove version from installed_crate_registry list so can be
// used for old clean flag
fn remove_version(installed_crate_registry: &[String]) -> Vec<(String, String)> {
    let mut removed_version = Vec::new();
    installed_crate_registry.iter().for_each(|crate_full_name| {
        let data = clear_version_value(crate_full_name);
        removed_version.push(data);
    });
    removed_version
}

// get latest commit rev value from git repository
fn latest_rev_value(path: &Path) -> String {
    let output = std::process::Command::new("git")
        .arg("log")
        .arg("--pretty=format:'%h'")
        .arg("--max-count=1")
        .current_dir(path)
        .output()
        .expect("failed to execute process");
    std::str::from_utf8(&output.stdout)
        .expect("stdout is not utf8")
        .to_string()
}
