use crate::{config_file::ConfigFile, crate_detail::CrateDetail, dir_path::DirPath};
use fs_extra::dir::get_size;
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

// struct store Cargo.lock file location
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
        // list out installed crates
        let installed_bin = get_installed_bin(bin_dir, crate_detail);
        let installed_crate_registry =
            get_installed_crate_registry(src_dir, cache_dir, crate_detail);
        let installed_crate_git = get_installed_crate_git(checkout_dir, db_dir, crate_detail);

        // list old registry crate
        let mut old_crate_registry = Vec::new();
        let mut version_removed_crate = remove_version(&installed_crate_registry);
        version_removed_crate.sort();
        if !version_removed_crate.is_empty() {
            let mut common_crate_version = Vec::new();
            for i in 0..(version_removed_crate.len() - 1) {
                let (name, version) = &version_removed_crate[i];
                let (next_name, _) = &version_removed_crate[i + 1];
                if name == next_name {
                    common_crate_version.push((version, i));
                } else {
                    common_crate_version.push((version, i));
                    let mut latest_version = version;
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
                    common_crate_version = vec![]
                }
            }
        }
        old_crate_registry.sort();
        old_crate_registry.dedup();

        // list old git crate
        let mut old_crate_git = Vec::new();
        for crate_name in &installed_crate_git {
            if !crate_name.contains("-HEAD") {
                let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let mut full_name_list = Vec::new();
                for crates in fs::read_dir(db_dir).expect("failed to read db dir") {
                    let entry = crates.unwrap().path();
                    let path = entry.as_path();
                    let file_name = path
                        .file_name()
                        .expect("failed to get file name form db directory sub folder");
                    let file_name = file_name.to_str().unwrap().to_string();
                    if file_name.contains(name[1]) {
                        let mut rev_value = latest_rev_value(path);
                        rev_value.retain(|c| c != '\'');
                        let full_name = format!("{}-{}", name[1], rev_value);
                        full_name_list.push(full_name)
                    }
                }
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
        let mut env_directory = env_list("TRIM_DIRECTORY");
        let mut config_directory = config_file.directory().to_owned();
        env_directory.append(&mut config_directory);
        env_directory.sort();
        env_directory.dedup();
        for path in &env_directory {
            let list_cargo_toml = list_cargo_toml(Path::new(path));
            let (mut registry_crate, mut git_crate) =
                read_content(list_cargo_toml.location_path(), db_dir);
            cargo_toml_location.append(list_cargo_toml);
            used_crate_registry.append(&mut registry_crate);
            used_crate_git.append(&mut git_crate);
        }
        used_crate_registry.sort();
        used_crate_registry.dedup();
        used_crate_git.sort();
        used_crate_registry.dedup();

        // list orphan crates
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

// remove version tag from crates full tag mini function of remove_version
// function
fn clear_version_value(full_name: &str) -> (String, String) {
    let build_number_split: Vec<&str> = full_name.rsplitn(2, '+').collect();
    let splitted_name = if build_number_split.len() == 1 {
        build_number_split[0]
    } else {
        build_number_split[1]
    };
    let list: Vec<&str> = splitted_name.rsplitn(3, '-').collect();
    let mut clear_name = String::new();
    let mut version = if semver::Version::parse(list[0]).is_ok() {
        for (i, a) in list[1..].iter().rev().enumerate() {
            clear_name.push_str(a);
            if i != list.len() - 2 {
                clear_name.push_str("-");
            }
        }
        list[0].to_string()
    } else {
        clear_name = list[2].to_string();
        let mut version = String::new();
        version.push_str(list[0]);
        version.push_str("-");
        version.push_str(list[1]);
        version
    };
    if build_number_split.len() != 1 {
        version.push_str("+");
        version.push_str(build_number_split[0]);
    }
    (clear_name, version)
}

// List out cargo.toml file present directory inside directory listed inside
// config file
fn list_cargo_toml(path: &Path) -> CargoTomlLocation {
    let mut list = CargoTomlLocation::new();
    if path.exists() {
        for entry in std::fs::read_dir(path)
            .expect("failed to read directory while trying to find cargo.toml")
        {
            let data_path_buf = entry.unwrap().path();
            let data = data_path_buf.as_path();
            if data.is_dir() && !data.file_name().unwrap().to_str().unwrap().starts_with('.') {
                let kids_list = list_cargo_toml(data);
                list.append(kids_list);
            }
            if data.is_file() && data.ends_with("Cargo.toml") {
                list.add_path(path.to_path_buf());
            }
        }
    }
    list
}

// Read out content of cargo.lock file to list out crates present so can be used
// for orphan clean
fn read_content(list: &[PathBuf], db_dir: &Path) -> (Vec<String>, Vec<String>) {
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
                            let mut path_db_list = Vec::new();
                            for git_db in fs::read_dir(db_dir).expect("failed to read git db dir") {
                                let entry = git_db.unwrap().path();
                                let file_name = entry
                                    .file_name()
                                    .expect("failed to get file name")
                                    .to_str()
                                    .expect("failed to convert OS str to str");
                                if file_name.contains(name) {
                                    let mut path_db = db_dir.to_path_buf();
                                    path_db.push(&file_name);
                                    path_db_list.push(path_db);
                                }
                            }
                            for path_db in path_db_list {
                                if source.contains("?rev=") {
                                    let rev: Vec<&str> = source.split("?rev=").collect();
                                    let rev_sha: Vec<&str> = rev[1].split('#').collect();
                                    let rev_value = rev_sha[1].to_string();
                                    let rev_short_form = &rev_value[..=6];
                                    let full_name = format!("{}-{}", name, rev_short_form);
                                    present_crate_git.push(full_name);
                                } else if source.contains("?branch=") || source.contains("?tag=") {
                                    let branch: Vec<&str> = if source.contains("?branch=") {
                                        source.split("?branch=").collect()
                                    } else {
                                        source.split("?tag=").collect()
                                    };
                                    let branch: Vec<&str> = branch[1].split('#').collect();
                                    let branch_value = branch[0];
                                    let output = std::process::Command::new("git")
                                        .arg("log")
                                        .arg("--pretty=format:%h")
                                        .arg("--max-count=1")
                                        .arg(branch_value)
                                        .current_dir(path_db)
                                        .output()
                                        .expect(
                                            "failed to execute command for pretty log of branch",
                                        );
                                    let rev_value = std::str::from_utf8(&output.stdout)
                                        .expect("stdout is not utf8");
                                    let full_name = format!("{}-{}", name, rev_value);
                                    present_crate_git.push(full_name);
                                } else {
                                    let rev_value = latest_rev_value(&path_db);
                                    let full_name = format!("{}-{}", name, rev_value);
                                    present_crate_git.push(full_name);
                                }
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
    for i in installed_crate_registry.iter() {
        let data = clear_version_value(i);
        removed_version.push(data);
    }
    removed_version
}

// list out installed bin
fn get_installed_bin(bin_dir: &Path, crate_detail: &mut CrateDetail) -> Vec<String> {
    let mut installed_bin = Vec::new();
    if bin_dir.exists() {
        for entry in fs::read_dir(bin_dir).expect("failed to read bin directory") {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let bin_size = get_size(&path).expect("failed to get size of bin directory");
            let file_name = path
                .file_name()
                .expect("failed to get file name from bin directory");
            let bin_name = file_name.to_str().unwrap().to_string();
            crate_detail.add_bin(bin_name.to_owned(), bin_size);
            installed_bin.push(bin_name)
        }
    }
    installed_bin.sort();
    installed_bin
}

// list out installed registry crates
fn get_installed_crate_registry(
    src_dir: &Path,
    cache_dir: &Path,
    crate_detail: &mut CrateDetail,
) -> Vec<String> {
    let mut installed_crate_registry = Vec::new();
    if src_dir.exists() {
        for entry in fs::read_dir(src_dir).expect("failed to read src directory") {
            let registry = entry.unwrap().path();
            for entry in fs::read_dir(registry).expect("failed to read registry folder") {
                let entry = entry.unwrap().path();
                let path = entry.as_path();
                let crate_size = get_size(&path).expect("failed to get registry crate size");
                let file_name = path
                    .file_name()
                    .expect("failed to get file name form main entry");
                let crate_name = file_name.to_str().unwrap().to_string();
                crate_detail.add_registry_crate_source(crate_name.to_owned(), crate_size);
                installed_crate_registry.push(crate_name)
            }
        }
    }
    if cache_dir.exists() {
        for entry in fs::read_dir(cache_dir).expect("failed to read cache dir") {
            let registry = entry.unwrap().path();
            for entry in fs::read_dir(registry).expect("failed to read cache dir registry folder") {
                let entry = entry.unwrap().path();
                let path = entry.as_path();
                let file_name = path
                    .file_name()
                    .expect("failed to get file name from cache dir");
                let crate_size = get_size(&path).expect("failed to get size");
                let crate_name = file_name.to_str().unwrap().to_string();
                let splitted_name = crate_name.rsplitn(2, '.').collect::<Vec<&str>>();
                crate_detail.add_registry_crate_archive(splitted_name[1].to_owned(), crate_size);
                installed_crate_registry.push(splitted_name[1].to_owned());
            }
        }
    }
    installed_crate_registry.sort();
    installed_crate_registry.dedup();
    installed_crate_registry
}

// list out installed git crates
fn get_installed_crate_git(
    checkout_dir: &Path,
    db_dir: &Path,
    crate_detail: &mut CrateDetail,
) -> Vec<String> {
    let mut installed_crate_git = Vec::new();
    if checkout_dir.exists() {
        for entry in fs::read_dir(checkout_dir).expect("failed to read checkout directory") {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_path = path
                .file_name()
                .expect("failed to obtain checkout directory sub folder file name");
            for git_sha_entry in fs::read_dir(path).expect("failed to read checkout dir sub folder")
            {
                let git_sha_entry = git_sha_entry.unwrap().path();
                let git_sha_path = git_sha_entry.as_path();
                let crate_size = get_size(git_sha_path).expect("failed to get folder size");
                let git_sha_file_name = git_sha_path.file_name().expect("failed to get file name");
                let git_sha = git_sha_file_name.to_str().unwrap().to_string();
                let file_name = file_path.to_str().unwrap().to_string();
                let splitted_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let full_name = format!("{}-{}", splitted_name[1], git_sha);
                crate_detail.add_git_crate_archive(full_name.to_owned(), crate_size);
                installed_crate_git.push(full_name)
            }
        }
    }
    if db_dir.exists() {
        for entry in fs::read_dir(db_dir).expect("failed to read db dir") {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let crate_size = get_size(path).expect("failed to get size of db dir folders");
            let file_name = path.file_name().expect("failed to get file name");
            let file_name = file_name.to_str().unwrap().to_string();
            let splitted_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
            let full_name = format!("{}-HEAD", splitted_name[1]);
            crate_detail.add_git_crate_source(full_name.to_owned(), crate_size);
            installed_crate_git.push(full_name);
        }
    }
    installed_crate_git.sort();
    installed_crate_git.dedup();
    installed_crate_git
}

// list out a env variables list in vector form
pub(crate) fn env_list(variable: &str) -> Vec<String> {
    let list = env::var(variable);
    let mut vec_list = Vec::new();
    if let Ok(name_list) = list {
        for name in name_list.split_whitespace() {
            vec_list.push(name.to_string());
        }
    }
    vec_list
}

// get out latest commit rev value
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
