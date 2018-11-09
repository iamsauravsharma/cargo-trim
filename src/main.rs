use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};

mod create_app;

struct GitDir {
    git_cache_dir: String,
    git_src_dir: String,
}

impl GitDir {
    fn new(cache_dir: &Path, src_dir: &Path) -> GitDir {
        let git_cache_dir = open_github_folder(&cache_dir).unwrap();
        let git_src_dir = open_github_folder(&src_dir).unwrap();
        GitDir {
            git_cache_dir,
            git_src_dir,
        }
    }

    fn remove_crate(&self, crate_name: &str) {
        remove_value(Path::new(&self.git_cache_dir), crate_name);
        remove_value(Path::new(&self.git_src_dir), crate_name);
    }
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    directory: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
}

impl ConfigFile {
    fn new() -> ConfigFile {
        ConfigFile {
            directory: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }
}

fn main() {
    let mut config_dir = dirs::config_dir().unwrap();
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".cargo");
    home_dir.push("registry");
    config_dir.push("cargo_cache_config.json");
    if !config_dir.exists() {
        fs::File::create(config_dir.to_str().unwrap()).unwrap();
    }
    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();

    let config_file = write_to_file(&mut file, &app, &config_dir);

    let mut cache_dir = home_dir.clone();
    cache_dir.push("cache");
    let mut src_dir = home_dir.clone();
    src_dir.push("src");
    let gitdir = GitDir::new(&cache_dir, &src_dir);
    let mut installed_crate = list_crate(Path::new(&gitdir.git_src_dir));
    installed_crate.sort();
    let read_include = config_file.include;
    let read_exculde = config_file.exclude;

    if app.is_present("list") {
        for list in &installed_crate {
            println!("{}", list);
        }
    }

    if app.is_present("old clean") {
        let mut old_version = Vec::new();
        let mut version_removed_crate = remove_version(&installed_crate);
        version_removed_crate.sort();
        for i in 0..(version_removed_crate.len() - 1) {
            if version_removed_crate[i] == version_removed_crate[i + 1] {
                old_version.push(&installed_crate[i])
            }
        }
        old_version.sort();
        for crate_name in &old_version {
            gitdir.remove_crate(crate_name);
        }
    }

    if app.is_present("clear config") {
        fs::remove_file(Path::new(config_dir.to_str().unwrap())).unwrap();
    }

    let mut cmd_include = Vec::new();
    let mut cmd_exclude = Vec::new();
    if app.is_present("include") {
        let value = app.value_of("include").unwrap().to_string();
        cmd_include.push(value);
    }
    if app.is_present("exclude") {
        let value = app.value_of("include").unwrap().to_string();
        cmd_exclude.push(value);
    }

    if app.is_present("remove") {
        let value = app.value_of("remove").unwrap();
        gitdir.remove_crate(value);
    }

    if app.is_present("force remove") {
        fs::remove_dir_all(home_dir).unwrap();
    }

    if app.is_present("all") {
        for crate_name in &installed_crate {
            if cmd_include.contains(crate_name) || read_include.contains(crate_name) {
                gitdir.remove_crate(crate_name);
            }
            if !cmd_exclude.contains(crate_name) && !read_exculde.contains(crate_name) {
                gitdir.remove_crate(crate_name);
            }
        }
    }
}

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

fn list_crate(src_dir: &Path) -> Vec<String> {
    let mut list = Vec::new();
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap().path();
        let path = entry.as_path();
        let file_name = path.file_name().unwrap();
        let crate_name = file_name.to_str().unwrap().to_string();
        list.push(crate_name)
    }
    list
}

fn open_github_folder(path: &Path) -> Option<String> {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();
        if path.contains("github.com") {
            return Some(path.to_string());
        }
    }
    None
}

fn remove_value(path: &Path, value: &str) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.to_str().unwrap().contains(value) {
            if path.is_file() {
                fs::remove_file(path).unwrap();
            } else if path.is_dir() {
                fs::remove_dir_all(path).unwrap();
            }
        } else if path.is_dir() {
            remove_value(&path, value);
        }
    }
}

fn remove_version(installed_crate: &[String]) -> Vec<String> {
    let mut removed_version = Vec::new();
    for i in installed_crate.iter() {
        let data = clear_version_value(i);
        removed_version.push(data);
    }
    removed_version
}

fn write_to_file(file: &mut fs::File, app: &clap::ArgMatches, config_dir: &PathBuf) -> ConfigFile {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    if buffer.is_empty() {
        let initial_config = ConfigFile::new();
        let serialize = serde_json::to_string(&initial_config).unwrap();
        buffer.push_str(&serialize)
    }
    let mut deserialized: ConfigFile = serde_json::from_str(&buffer).unwrap();
    for &name in &["set-directory", "exclude-conf", "include-conf"] {
        if app.is_present(name) {
            let value = app.value_of(name).unwrap();
            if name == "set-directory" {
                deserialized.directory.push(value.to_string());
            }
            if name == "exclude-conf" {
                deserialized.exclude.push(value.to_string());
            }
            if name == "include-conf" {
                deserialized.include.push(value.to_string());
            }
        }
    }
    let serialized = serde_json::to_string(&deserialized).unwrap();
    buffer.clear();
    buffer.push_str(&serialized);
    fs::write(config_dir, buffer).unwrap();
    deserialized
}
