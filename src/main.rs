#![feature(vec_remove_item)]

mod config_file;
mod create_app;
mod git_dir;
mod list_crate;
#[cfg(test)]
mod test;

use fs_extra::dir as dir_extra;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let (config_dir, registry_dir, cache_dir, index_dir, src_dir) = create_dir();

    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();

    // Perform all modification of config file flag and subcommand operation
    let config_file = config_file::modify_config_file(&mut file, &app, &config_dir);

    let git_dir = git_dir::GitDir::new(&cache_dir, &src_dir);

    // List out crate list
    let list_crate = list_crate::CrateList::create_list(Path::new(git_dir.src()), &config_file);
    let installed_crate = list_crate.installed();
    let old_crate = list_crate.old();
    let used_crate = list_crate.used();

    let read_include = config_file.include();
    let read_exclude = config_file.exclude();

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
        let metadata_registry = dir_extra::get_size(registry_dir.clone()).unwrap() as f64;
        let metadata_cache = dir_extra::get_size(cache_dir.clone()).unwrap() as f64;
        let metadata_index = dir_extra::get_size(index_dir.clone()).unwrap() as f64;
        let metadata_src = dir_extra::get_size(src_dir.clone()).unwrap() as f64;
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
        for crate_name in &old_crate {
            git_dir.remove_crate(crate_name);
            println!("Removed {:?}", crate_name);
        }
        println!("Successfully removed {:?} crates", old_crate.len());
    }

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file
    if app.is_present("orphan clean") {
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

fn create_dir() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let mut config_dir = dirs::config_dir().unwrap();
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".cargo");
    home_dir.push("registry");
    let registry_dir = home_dir;
    config_dir.push("cargo_cache_config.json");

    // If config file does not exists create one config file
    if !config_dir.exists() {
        fs::File::create(config_dir.to_str().unwrap()).unwrap();
    }

    let mut cache_dir = registry_dir.clone();
    cache_dir.push("cache");
    let mut src_dir = registry_dir.clone();
    src_dir.push("src");
    let mut index_dir = registry_dir.clone();
    index_dir.push("index");

    (config_dir, registry_dir, cache_dir, index_dir, src_dir)
}
