use crate::config_file::ConfigFile;
use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

pub(super) struct CrateList {
    installed_crate: Vec<String>,
    old_crate: Vec<String>,
    used_crate: Vec<String>,
}

impl CrateList {
    pub(super) fn create_list(src_dir: &Path, config_file: &ConfigFile) -> Self {
        let mut installed_crate = Vec::new();
        for entry in fs::read_dir(src_dir).unwrap() {
            let entry = entry.unwrap().path();
            let path = entry.as_path();
            let file_name = path.file_name().unwrap();
            let crate_name = file_name.to_str().unwrap().to_string();
            installed_crate.push(crate_name)
        }
        installed_crate.sort();
        let mut old_crate = Vec::new();
        let mut version_removed_crate = remove_version(&installed_crate);
        version_removed_crate.sort();
        for i in 0..(version_removed_crate.len() - 1) {
            if version_removed_crate[i] == version_removed_crate[i + 1] {
                let old_crate_name = installed_crate[i].to_string();
                old_crate.push(old_crate_name);
            }
        }
        old_crate.sort();
        let mut used_crate = Vec::new();
        for path in config_file.directory().iter() {
            let list = list_cargo_lock(&Path::new(path));
            let mut buffer_crate = read_content(&list);
            used_crate.append(&mut buffer_crate);
        }
        Self {
            installed_crate,
            old_crate,
            used_crate,
        }
    }

    pub(super) fn installed(&self) -> Vec<String> {
        self.installed_crate.to_owned()
    }

    pub(super) fn old(&self) -> Vec<String> {
        self.old_crate.to_owned()
    }

    pub(super) fn used(&self) -> Vec<String> {
        self.used_crate.to_owned()
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
fn read_content(list: &[PathBuf]) -> Vec<String> {
    let mut present_crate = Vec::new();
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
                let full_name = format!("{}-{}", name, version);
                present_crate.push(full_name);
            }
        }
    }
    present_crate
}

// Function used to remove version from installed_crate list so can be used for
// old clean flag
fn remove_version(installed_crate: &[String]) -> Vec<String> {
    let mut removed_version = Vec::new();
    for i in installed_crate.iter() {
        let data = clear_version_value(i);
        removed_version.push(data);
    }
    removed_version
}
