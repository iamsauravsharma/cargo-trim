#![feature(vec_remove_item)]

mod config_file;
mod create_app;
mod create_dir;
mod git_dir;

use fs_extra::dir;
use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

fn main() {
    let (config_dir, registry_dir, cache_dir, index_dir, src_dir) = create_dir::create_dir();

    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();

    let config_file = config_file::modify_config_file(&mut file, &app, &config_dir);

    let git_dir = git_dir::GitDir::new(&cache_dir, &src_dir);

    // List out installed crate list
    let mut installed_crate = list_crate(Path::new(git_dir.get_src()));
    installed_crate.sort();

    let read_include = config_file.get_include();
    let read_exclude = config_file.get_exclude();

    // Perform action of removing config file with -c flag
    if app.is_present("clear config") {
        fs::remove_file(Path::new(config_dir.to_str().unwrap())).unwrap();
        println!("Cleared Config file");
    }

    // Perform action on list subcommand
    if app.is_present("list") {
        for list in &installed_crate {
            println!("{}", list);
        }
    }

    // Perform action for -q flag
    if app.is_present("query size") {
        let metadata_registry = dir::get_size(registry_dir.clone()).unwrap() as f64;
        let metadata_cache = dir::get_size(cache_dir.clone()).unwrap() as f64;
        let metadata_index = dir::get_size(index_dir.clone()).unwrap() as f64;
        let metadata_src = dir::get_size(src_dir.clone()).unwrap() as f64;
        println!(
            "{:50} {:.3} MB",
            format!("Size of {} .cargo/registry crates:", installed_crate.len()),
            metadata_registry / (1024f64.powf(2.0))
        );
        println!(
            "{:50} {:.3} MB",
            "   |-- Size of .cargo/registry/cache folder",
            metadata_cache / (1024f64.powf(2.0))
        );

        println!(
            "{:50} {:.3} MB",
            "   |-- Size of .cargo/registry/index folder",
            metadata_index / (1024f64.powf(2.0))
        );
        println!(
            "{:50} {:.3} MB",
            "   |-- Size of .cargo/registry/src folder",
            metadata_src / (1024f64.powf(2.0))
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
        for path in config_file.get_directory().iter() {
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
    if app.is_present("remove-crate") {
        let value = app.value_of("remove-crate").unwrap();
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
        fs::remove_dir_all(registry_dir.clone()).unwrap();
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

    if app.is_present("wipe") {
        let value = app.value_of("wipe").unwrap();
        match value {
            "registry" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            "cache" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            "index" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            "src" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            _ => println!("Please provide one of the given four value registry, cache, index, src"),
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

    #[test]
    fn test_list_help() {
        if !cfg!(target_os = "windows") {
            let output = Command::new("sh")
                .arg("-c")
                .arg("cargo run -- help list")
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8(output.stdout).unwrap();
            let mut buffer = String::new();
            let mut file = std::fs::File::open("tests/command_output/list.txt").unwrap();
            file.read_to_string(&mut buffer).unwrap();
            assert_eq!(output, buffer);
        }
    }

    #[test]
    fn test_remove_help() {
        if !cfg!(target_os = "windows") {
            let output = Command::new("sh")
                .arg("-c")
                .arg("cargo run -- help remove")
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8(output.stdout).unwrap();
            let mut buffer = String::new();
            let mut file = std::fs::File::open("tests/command_output/remove.txt").unwrap();
            file.read_to_string(&mut buffer).unwrap();
            assert_eq!(output, buffer);
        }
    }
}
