use crate::config_file::ConfigFile;
use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

pub(super) struct CrateList {
    installed_crate_registry: Vec<String>,
    installed_crate_git: Vec<String>,
    old_crate: Vec<String>,
    used_crate_registry: Vec<String>,
    used_crate_git: Vec<String>,
    orphan_crate_registry: Vec<String>,
    orphan_crate_git: Vec<String>,
}

impl CrateList {
    // create list of all types of crate present in directory
    pub(super) fn create_list(
        src_dir: &Path,
        checkout_dir: &Path,
        config_file: &ConfigFile,
    ) -> Self {
        let mut installed_crate_registry = Vec::new();
        for entry in fs::read_dir(src_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_name = path.file_name().unwrap();
            let crate_name = file_name.to_str().unwrap().to_string();
            installed_crate_registry.push(crate_name)
        }
        installed_crate_registry.sort();

        let mut installed_crate_git = Vec::new();
        for entry in fs::read_dir(checkout_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_name = path.file_name().unwrap();
            let file_name = file_name.to_str().unwrap().to_string();
            let splitted_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
            installed_crate_git.push(splitted_name[1].to_owned());
        }
        installed_crate_git.sort();

        let mut old_crate = Vec::new();
        let mut version_removed_crate = remove_version(&installed_crate_registry);
        version_removed_crate.sort();
        for i in 0..(version_removed_crate.len() - 1) {
            if version_removed_crate[i] == version_removed_crate[i + 1] {
                let old_crate_name = installed_crate_registry[i].to_string();
                old_crate.push(old_crate_name);
            }
        }
        old_crate.sort();

        let mut used_crate_registry = Vec::new();
        let mut used_crate_git = Vec::new();
        for path in config_file.directory().iter() {
            let list = list_cargo_lock(&Path::new(path));
            let (mut registry_crate, mut git_crate) = read_content(&list);
            used_crate_registry.append(&mut registry_crate);
            used_crate_git.append(&mut git_crate);
        }

        let mut orphan_crate_registry = Vec::new();
        let mut orphan_crate_git = Vec::new();
        for crates in &installed_crate_registry {
            if !used_crate_registry.contains(crates) {
                orphan_crate_registry.push(crates.to_string());
            }
        }
        for crates in &installed_crate_git {
            if !used_crate_git.contains(crates) {
                orphan_crate_git.push(crates.to_string());
            }
        }
        Self {
            installed_crate_registry,
            installed_crate_git,
            old_crate,
            used_crate_registry,
            used_crate_git,
            orphan_crate_registry,
            orphan_crate_git,
        }
    }

    pub(super) fn installed_registry(&self) -> Vec<String> {
        self.installed_crate_registry.to_owned()
    }

    pub(super) fn old_registry(&self) -> Vec<String> {
        self.old_crate.to_owned()
    }

    pub(super) fn used_registry(&self) -> Vec<String> {
        self.used_crate_registry.to_owned()
    }

    pub(super) fn orphan_registry(&self) -> Vec<String> {
        self.orphan_crate_registry.to_owned()
    }

    pub(super) fn installed_git(&self) -> Vec<String> {
        self.installed_crate_git.to_owned()
    }

    pub(super) fn used_git(&self) -> Vec<String> {
        self.used_crate_git.to_owned()
    }

    pub(super) fn orphan_git(&self) -> Vec<String> {
        self.orphan_crate_git.to_owned()
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

// List out cargo.lock file present inside directory listed inside config file
fn list_cargo_lock(path: &Path) -> Vec<PathBuf> {
    let mut list = Vec::new();
    for entry in std::fs::read_dir(path).expect("error 1") {
        let data = entry.unwrap().path();
        let data = data.as_path();
        if data.is_dir() {
            let mut kids_list = list_cargo_lock(data);
            list.append(&mut kids_list);
        }
        if data.is_file() && data.ends_with("Cargo.lock") {
            list.push(data.to_path_buf());
        }
    }
    list
}

// Read out content of cargo.lock file to list out crates present so can be used
// for orphan clean
fn read_content(list: &[PathBuf]) -> (Vec<String>, Vec<String>) {
    let mut present_crate_registry = Vec::new();
    let mut present_crate_git = Vec::new();
    for lock_file in list.iter() {
        let lock_file = lock_file.to_str().unwrap();
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
                if source.contains("(registry+") {
                    let full_name = format!("{}-{}", name, version);
                    present_crate_registry.push(full_name);
                }
                if source.contains("(git+") {
                    present_crate_git.push(name.to_string());
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
