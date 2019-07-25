use crate::{config_file::ConfigFile, crate_detail::CrateDetail};
use fs_extra::dir::get_size;
use std::{
    env, fs,
    io::prelude::*,
    path::{Path, PathBuf},
    process::Command,
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
        bin_dir: &Path,
        cache_dir: &Path,
        src_dir: &Path,
        checkout_dir: &Path,
        db_dir: &Path,
        config_file: &ConfigFile,
        crate_detail: &mut CrateDetail,
    ) -> Self {
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
            for i in 0..(version_removed_crate.len() - 1) {
                if version_removed_crate[i] == version_removed_crate[i + 1] {
                    let old_crate_name = installed_crate_registry[i].to_string();
                    old_crate_registry.push(old_crate_name);
                }
            }
        }
        old_crate_registry.sort();

        // list old git crate
        let mut old_crate_git = Vec::new();
        for crate_name in &installed_crate_git {
            if !crate_name.contains("-HEAD") {
                let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let mut full_name_list = Vec::new();
                for crates in fs::read_dir(db_dir).unwrap() {
                    let entry = crates.unwrap().path();
                    let path = entry.as_path();
                    let file_name = path.file_name().unwrap();
                    let file_name = file_name.to_str().unwrap().to_string();
                    if file_name.contains(name[1]) {
                        let output = std::process::Command::new("git")
                            .arg("log")
                            .arg("--pretty=format:'%h'")
                            .arg("--max-count=1")
                            .current_dir(path)
                            .output()
                            .unwrap();
                        let mut rev_value =
                            std::str::from_utf8(&output.stdout).unwrap().to_string();
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
            let list_cargo_toml = list_cargo_toml(&Path::new(path));
            let (mut registry_crate, mut git_crate) =
                read_content(&list_cargo_toml.location_path(), db_dir);
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
                for used in &used_crate_git {
                    if !used.contains(split_installed[1]) {
                        orphan_crate_git.push(crates.to_string())
                    }
                }
            } else if !used_crate_git.contains(crates) {
                orphan_crate_git.push(crates.to_string());
            }
        }
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
fn clear_version_value(a: &str) -> String {
    let list = a.rsplitn(2, '-');
    let mut value = String::new();
    for (i, val) in list.enumerate() {
        if i == 1 {
            value = val.to_string()
        }
    }
    value
}

// List out cargo.toml file present directory inside directory listed inside
// config file
fn list_cargo_toml(path: &Path) -> CargoTomlLocation {
    let mut list = CargoTomlLocation::new();
    if path.exists() {
        for entry in std::fs::read_dir(path).unwrap() {
            let data_path_buf = entry.unwrap().path();
            let data = data_path_buf.as_path();
            if data.is_dir() {
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
        // try generating lock file from Cargo.toml file location to guarantee this path
        // should have lock file or not
        if !lock_folder.exists() {
            if let Err(e) = Command::new("cargo")
                .arg("generate-lockfile")
                .current_dir(lock)
                .output()
            {
                panic!(format!("Failed to generate Cargo.lock {}", e));
            }
        }
        if lock_folder.exists() {
            let lock_file = lock_folder.to_str().unwrap();
            let mut buffer = String::new();
            let mut file = std::fs::File::open(lock_file).unwrap();
            file.read_to_string(&mut buffer).unwrap();
            let mut set_flag = 0;
            for line in buffer.lines() {
                if line.contains("[metadata]") {
                    set_flag = 1;
                    continue;
                }
                if set_flag == 1 {
                    let mut split = line.split_whitespace();
                    split.next();
                    let name = split.next().unwrap();
                    let version = split.next().unwrap();
                    let source = split.next().unwrap();
                    if source.contains("registry+") {
                        let full_name = format!("{}-{}", name, version);
                        present_crate_registry.push(full_name);
                    }
                    if source.contains("git+") {
                        let mut path_db = db_dir.to_path_buf();
                        path_db.push(name);
                        if source.contains("?rev=") {
                            let rev: Vec<&str> = line.split("?rev=").collect();
                            let rev_sha: Vec<&str> = rev[1].split(')').collect();
                            let rev_value = rev_sha[1].to_string();
                            let rev_short_form = &rev_value[..=6];
                            let full_name = format!("{}-{}", name, rev_short_form);
                            present_crate_git.push(full_name);
                        } else if source.contains("?branch=") || source.contains("?tag=") {
                            let branch: Vec<&str> = if source.contains("?branch=") {
                                line.split("?branch=").collect()
                            } else {
                                line.split("?tag=").collect()
                            };
                            let branch: Vec<&str> = branch[1].split(')').collect();
                            let branch_value = branch[1];
                            let output = std::process::Command::new("git")
                                .arg("log")
                                .arg("--pretty=format:'%h'")
                                .arg("--max-count=1")
                                .arg(branch_value)
                                .current_dir(path_db)
                                .output()
                                .unwrap();
                            let rev_value = std::str::from_utf8(&output.stdout).unwrap();
                            let full_name = format!("{}-{}", name, rev_value);
                            present_crate_git.push(full_name);
                        } else {
                            let output = std::process::Command::new("git")
                                .arg("log")
                                .arg("--pretty=format:'%h'")
                                .arg("--max-count=1")
                                .current_dir(path_db)
                                .output()
                                .unwrap();
                            let rev_value = std::str::from_utf8(&output.stdout).unwrap();
                            let full_name = format!("{}-{}", name, rev_value);
                            present_crate_git.push(full_name);
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
fn remove_version(installed_crate_registry: &[String]) -> Vec<String> {
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
        for entry in fs::read_dir(bin_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let bin_size = get_size(&path).unwrap();
            let file_name = path.file_name().unwrap();
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
        for entry in fs::read_dir(src_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let crate_size = get_size(&path).unwrap();
            let file_name = path.file_name().unwrap();
            let crate_name = file_name.to_str().unwrap().to_string();
            crate_detail.add_registry_crate_source(crate_name.to_owned(), crate_size);
            installed_crate_registry.push(crate_name)
        }
    }
    if cache_dir.exists() {
        for entry in fs::read_dir(cache_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_name = path.file_name().unwrap();
            let crate_size = get_size(&path).unwrap();
            let crate_name = file_name.to_str().unwrap().to_string();
            let splitted_name = crate_name.rsplitn(2, '.').collect::<Vec<&str>>();
            crate_detail.add_registry_crate_archive(splitted_name[1].to_owned(), crate_size);
            installed_crate_registry.push(splitted_name[1].to_owned());
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
        for entry in fs::read_dir(checkout_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_path = path.file_name().unwrap();
            for git_sha_entry in fs::read_dir(path).unwrap() {
                let git_sha_entry = git_sha_entry.unwrap().path();
                let git_sha_path = git_sha_entry.as_path();
                let crate_size = get_size(git_sha_path).unwrap();
                let git_sha_file_name = git_sha_path.file_name().unwrap();
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
        for entry in fs::read_dir(db_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let crate_size = get_size(path).unwrap();
            let file_name = path.file_name().unwrap();
            let file_name = file_name.to_str().unwrap().to_string();
            let full_name = format!("{}-HEAD", file_name);
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
