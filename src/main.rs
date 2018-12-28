mod config_file;
mod create_app;
mod dir_path;
mod git_dir;
mod list_crate;
mod registry_dir;
#[cfg(test)]
mod test;

use crate::{
    config_file::ConfigFile, dir_path::DirPath, list_crate::CrateList, registry_dir::RegistryDir,
};
use clap::ArgMatches;
use fs_extra::{dir::get_size, error::Error};
use std::{fs, path::Path};

fn main() {
    let dir_path = DirPath::set_dir_path();

    let config_dir = dir_path.config_dir();
    let git_dir = dir_path.git_dir();
    let checkout_dir = dir_path.checkout_dir();
    let db_dir = dir_path.db_dir();
    let registry_dir = dir_path.registry_dir();
    let cache_dir = dir_path.cache_dir();
    let index_dir = dir_path.index_dir();
    let src_dir = dir_path.src_dir();

    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();
    let app = app.subcommand_matches("trim").unwrap();

    // Perform all modification of config file flag and subcommand operation
    let config_file = config_file::modify_config_file(&mut file, app, &config_dir);

    let registry_crates_location = registry_dir::RegistryDir::new(&cache_dir, &src_dir);
    let git_crates_location = git_dir::GitDir::new(&checkout_dir, &db_dir);

    // List out crate list
    let list_crate = list_crate::CrateList::create_list(
        Path::new(registry_crates_location.src()),
        checkout_dir.as_path(),
        &config_file,
    );
    let installed_registry_crate = list_crate.installed_registry();
    let old_registry_crate = list_crate.old_registry();
    let orphan_registry_crate = list_crate.orphan_registry();

    let installed_git_crate = list_crate.installed_git();
    let orphan_git_crate = list_crate.orphan_git();

    // Perform action of removing config file with -c flag
    if app.is_present("clear config") {
        fs::remove_file(Path::new(config_dir.to_str().unwrap())).unwrap();
        println!("Cleared Config file");
    }

    // Perform action on list subcommand
    if app.is_present("list") {
        let matches = app.subcommand_matches("list").unwrap();
        list_subcommand(matches, list_crate)
    }

    // Perform action for -q flag
    if app.is_present("query size") {
        let metadata_git = match_size(&get_size(git_dir.clone()));
        let metadata_checkout = match_size(&get_size(checkout_dir.clone()));
        let metadata_db = match_size(&get_size(db_dir.clone()));
        let metadata_registry = match_size(&get_size(registry_dir.clone()));
        let metadata_cache = match_size(&get_size(cache_dir.clone()));
        let metadata_index = match_size(&get_size(index_dir.clone()));
        let metadata_src = match_size(&get_size(src_dir.clone()));
        println!(
            "{:50} {:.3} MB",
            "Total Size of .cargo/git crates:",
            metadata_git / (1024f64.powf(2.0))
        );
        println!(
            "{:50} {:.3} MB",
            "   |-- Size of .cargo/git/checkout folder",
            metadata_checkout / (1024f64.powf(2.0))
        );
        println!(
            "{:50} {:.3} MB",
            "   |--    Size of .cargo/git/db folder",
            metadata_db / (1024f64.powf(2.0))
        );
        println!(
            "{:50} {:.3} MB",
            "Total Size of .cargo/registry crates:",
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
        for crate_name in &old_registry_crate {
            registry_crates_location.remove_crate(crate_name);
        }
        println!("Successfully removed {:?} crates", old_registry_crate.len());
    }

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file
    if app.is_present("orphan clean") {
        for crate_name in &orphan_registry_crate {
            registry_crates_location.remove_crate(crate_name);
        }
        println!(
            "Successfully removed {:?} crates",
            orphan_registry_crate.len() + orphan_git_crate.len()
        );
    }

    // Remove certain crate provided with -r flag
    if app.is_present("remove-crate") {
        let value = app.value_of("remove-crate").unwrap();
        if installed_registry_crate.contains(&value.to_string()) {
            registry_crates_location.remove_crate(value)
        }

        if installed_git_crate.contains(&value.to_string()) {
            git_crates_location.remove_crate(value)
        }
    }

    // Force remove all crates without reading config file
    if app.is_present("force remove") {
        fs::remove_dir_all(cache_dir.clone()).unwrap();
        fs::remove_dir_all(src_dir.clone()).unwrap();
        fs::remove_dir_all(checkout_dir.clone()).unwrap();
        fs::remove_dir_all(db_dir.clone()).unwrap();
    }

    // Remove all crates by following config file
    if app.is_present("all") {
        for crate_name in &installed_registry_crate {
            remove_registry_all(&config_file, app, crate_name, &registry_crates_location);
        }
    }

    // Wipe certain folder all together
    if app.is_present("wipe") {
        let value = app.value_of("wipe").unwrap();
        match value {
            "git" => fs::remove_dir_all(git_dir.clone()).unwrap(),
            "checkouts" => fs::remove_dir_all(checkout_dir.clone()).unwrap(),
            "db" => fs::remove_dir_all(db_dir.clone()).unwrap(),
            "registry" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            "cache" => fs::remove_dir_all(cache_dir.clone()).unwrap(),
            "index" => fs::remove_dir_all(index_dir.clone()).unwrap(),
            "src" => fs::remove_dir_all(src_dir.clone()).unwrap(),
            _ => println!("Please provide one of the given value"),
        }
    }

    // Query about config file information
    if app.is_present("query") {
        let matches = app.subcommand_matches("query").unwrap();
        query_subcommand(&config_file, matches)
    }
}

fn match_size(size: &Result<u64, Error>) -> f64 {
    match *size {
        Ok(size) => size as f64,
        Err(_) => 0.0,
    }
}

fn list_subcommand(list_subcommand: &ArgMatches, list_crate: CrateList) {
    if list_subcommand.is_present("old") {
        for crates in &list_crate.old_registry() {
            println!("{}", crates);
        }
    } else if list_subcommand.is_present("orphan") {
        for crates in &list_crate.orphan_registry() {
            println!("{}", crates);
        }
        for crates in &list_crate.orphan_git() {
            println!("{}", crates);
        }
    } else if list_subcommand.is_present("used") {
        for crates in &list_crate.used_registry() {
            println!("{}", crates);
        }
        for crates in &list_crate.used_git() {
            println!("{}", crates);
        }
    } else {
        for crates in &list_crate.installed_registry() {
            println!("{}", crates);
        }
        for crates in &list_crate.installed_git() {
            println!("{}", crates);
        }
    }
}

// Perform all query subcommand call operation
fn query_subcommand(config_file: &ConfigFile, matches: &ArgMatches) {
    let read_include = config_file.include();
    let read_exclude = config_file.exclude();
    let read_directory = config_file.directory();
    if matches.is_present("directory") {
        for name in &read_directory {
            println!("{}", name);
        }
    }
    if matches.is_present("include") {
        for name in &read_include {
            println!("{}", name);
        }
    }
    if matches.is_present("exclude") {
        for name in &read_exclude {
            println!("{}", name);
        }
    }
}

// Remove all crates from registry folder
fn remove_registry_all(
    config_file: &ConfigFile,
    app: &ArgMatches,
    crate_name: &str,
    registry_crates_location: &RegistryDir,
) {
    let mut cmd_include = Vec::new();
    let mut cmd_exclude = Vec::new();
    let crate_name = &crate_name.to_string();

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

    let read_include = config_file.include();
    let read_exclude = config_file.exclude();
    if cmd_include.contains(crate_name) || read_include.contains(crate_name) {
        registry_crates_location.remove_crate(crate_name);
    }
    if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
        registry_crates_location.remove_crate(crate_name);
    }
}
