use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};

mod create_app;

// Stores .cargo/registry cache & src information
struct GitDir {
    git_cache_dir: String,
    git_src_dir: String,
}

impl GitDir {
    // Create new GitDir
    fn new(cache_dir: &Path, src_dir: &Path) -> GitDir {
        let git_cache_dir = open_github_folder(&cache_dir).unwrap();
        let git_src_dir = open_github_folder(&src_dir).unwrap();
        GitDir {
            git_cache_dir,
            git_src_dir,
        }
    }

    // Remove crate from src & cache directory
    fn remove_crate(&self, crate_name: &str) {
        remove_crate(Path::new(&self.git_cache_dir), crate_name);
        remove_crate(Path::new(&self.git_src_dir), crate_name);
    }
}

// Stores config file information
#[derive(Serialize, Deserialize)]
struct ConfigFile {
    directory: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
}

impl ConfigFile {
    // Create new config file
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

    // If config file does not exists create one config file
    if !config_dir.exists() {
        fs::File::create(config_dir.to_str().unwrap()).unwrap();
    }
    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();

    let config_file = modify_config_file(&mut file, &app, &config_dir);

    let mut cache_dir = home_dir.clone();
    cache_dir.push("cache");
    let mut src_dir = home_dir.clone();
    src_dir.push("src");
    let git_dir = GitDir::new(&cache_dir, &src_dir);

    // List out installed crate list
    let mut installed_crate = list_crate(Path::new(&git_dir.git_src_dir));
    installed_crate.sort();

    let read_include = config_file.include;
    let read_exclude = config_file.exclude;

    // Perform action of removing config file with -c flag
    if app.is_present("clear config") {
        fs::remove_file(Path::new(config_dir.to_str().unwrap())).unwrap();
        println!("Cleared Config file");
    }

    // Perform action on -l flag
    if app.is_present("list") {
        for list in &installed_crate {
            println!("{}", list);
        }
    }

    // Perform action for -q flag
    if app.is_present("query size") {
        let metadata = fs_extra::dir::get_size(home_dir.clone()).unwrap() as f64;
        println!(
            "Size taken by .cargo/registry: {:.4} MB",
            metadata / (1024f64.powf(2.0))
        );
    }

    // Perform action on -o flag
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
            git_dir.remove_crate(crate_name);
            println!("Removed {:?}", crate_name);
        }
        println!("Successfully removed {:?} crates", old_version.len());
    }

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file
    if app.is_present("orphan clean") {
        let mut used_crate = Vec::new();
        for path in config_file.directory.iter() {
            let list = list_cargo_lock(&Path::new(path));
            let mut buffer_crate = read_content(&list);
            used_crate.append(&mut buffer_crate);
        }
        let mut count = 0;
        for crate_name in &installed_crate {
            if !used_crate.contains(crate_name) {
                git_dir.remove_crate(crate_name);
                count += 1;
                println!("Removed {:?}", crate_name);
            }
        }
        println!("Successfully removed {:?} crates", count);
    }

    // Remove certain crate provided with -r flag
    if app.is_present("remove") {
        let value = app.value_of("remove").unwrap();
        git_dir.remove_crate(value);
        println!("Removed {:?}", value);
    }

    let mut cmd_include = Vec::new();
    let mut cmd_exclude = Vec::new();

    // Provide one time include crate list for other flag
    if app.is_present("include") {
        let value = app.value_of("include").unwrap().to_string();
        cmd_include.push(value);
    }

    // Provide one time exclude crate list for other flag
    if app.is_present("exclude") {
        let value = app.value_of("include").unwrap().to_string();
        cmd_exclude.push(value);
    }

    // Force remove all crates without reading config file
    if app.is_present("force remove") {
        fs::remove_dir_all(home_dir.clone()).unwrap();
    }

    // Remove all crates by following config file
    if app.is_present("all") {
        for crate_name in &installed_crate {
            if cmd_include.contains(crate_name) || read_include.contains(crate_name) {
                git_dir.remove_crate(crate_name);
            }
            if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
                git_dir.remove_crate(crate_name);
            }
        }
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

// List out all crates present at .cache/registry
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

// Use to open github folder present inside src and cache folder
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

// Remove crates which name is provided to delete
fn remove_crate(path: &Path, value: &str) {
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
            remove_crate(&path, value);
        }
    }
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

// Function to modify config file or read config file
fn modify_config_file(
    file: &mut fs::File,
    app: &clap::ArgMatches,
    config_dir: &PathBuf,
) -> ConfigFile {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    if buffer.is_empty() {
        let initial_config = ConfigFile::new();
        let serialize = serde_json::to_string(&initial_config).unwrap();
        buffer.push_str(&serialize)
    }
    let mut deserialized: ConfigFile = serde_json::from_str(&buffer).unwrap();
    for &name in &["set directory", "exclude-conf", "include-conf"] {
        if app.is_present(name) {
            let value = app.value_of(name).unwrap();
            if name == "set directory" {
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

#[cfg(test)]
mod test {
    use std::{io::Read, process::Command};
    #[test]
    fn test_help() {
        if !cfg!(target_os = "windows") {
            let output = Command::new("sh")
                .arg("-c")
                .arg("cargo run -- -h")
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8(output.stdout).unwrap();
            let mut buffer = String::new();
            let mut file = std::fs::File::open("tests/command_output/help.txt").unwrap();
            file.read_to_string(&mut buffer).unwrap();
            assert_eq!(output, buffer);
        }
    }
}
