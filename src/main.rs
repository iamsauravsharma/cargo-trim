mod config_file;
mod create_app;
mod dir_path;
mod git_dir;
mod list_crate;
mod registry_dir;
#[cfg(test)]
mod test;

use crate::{
    config_file::ConfigFile, dir_path::DirPath, git_dir::GitDir, list_crate::CrateList,
    registry_dir::RegistryDir,
};
use clap::ArgMatches;
use fs_extra::dir::get_size;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let dir_path = DirPath::set_dir_path();
    let mut file = fs::File::open(dir_path.config_dir().to_str().unwrap()).unwrap();
    let app = create_app::app();
    let app = app.subcommand_matches("trim").unwrap();
    let mut git_subcommand = &ArgMatches::new();
    let mut registry_subcommand = &ArgMatches::new();
    if app.is_present("git") {
        git_subcommand = app.subcommand_matches("git").unwrap();
    }
    if app.is_present("registry") {
        registry_subcommand = app.subcommand_matches("registry").unwrap();
    }

    // Perform all modification of config file flag and subcommand operation
    let config_file = config_file::modify_config_file(&mut file, app, &dir_path.config_dir());

    let registry_crates_location =
        registry_dir::RegistryDir::new(&dir_path.cache_dir(), &dir_path.src_dir());
    let git_crates_location = git_dir::GitDir::new(&dir_path.checkout_dir(), &dir_path.db_dir());

    // List out crate list
    let list_crate = list_crate::CrateList::create_list(
        Path::new(registry_crates_location.src()),
        dir_path.checkout_dir().as_path(),
        &config_file,
    );

    // Perform action of removing config file with -c flag
    if app.is_present("clear config") {
        fs::remove_file(dir_path.config_dir().as_path()).unwrap();
        println!("Cleared Config file");
    }

    // Perform action on list subcommand
    if app.is_present("list") {
        let matches = app.subcommand_matches("list").unwrap();
        list_subcommand(matches, &list_crate)
    }

    // Perform action for -s flag
    let query_size_app = app.is_present("query size");
    let query_size_git = git_subcommand.is_present("query size");
    let query_size_registry = registry_subcommand.is_present("query size");
    query_size(
        &dir_path,
        &list_crate,
        (query_size_app, query_size_git, query_size_registry),
    );

    // Perform action on -o flag
    if app.is_present("old clean") || registry_subcommand.is_present("old clean") {
        let old_registry_crate = list_crate.old_registry();
        for crate_name in &old_registry_crate {
            registry_crates_location.remove_crate(crate_name);
        }
    }

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file
    let orphan_app = app.is_present("orphan clean");
    let orphan_git = git_subcommand.is_present("orphan clean");
    let orphan_registry = registry_subcommand.is_present("orphan clean");
    orphan_clean(
        &list_crate,
        (orphan_app, orphan_git, orphan_registry),
        &registry_crates_location,
        &git_crates_location,
    );

    // Remove certain crate provided with -r flag
    let remove_crate_app = app.is_present("remove-crate");
    let remove_crate_git = git_subcommand.is_present("remove-crate");
    let remove_crate_registry = registry_subcommand.is_present("remove-crate");
    if remove_crate_app || remove_crate_git || remove_crate_registry {
        let value = app.value_of("remove-crate").unwrap_or_else(|| {
            git_subcommand
                .value_of("remove-crate")
                .unwrap_or_else(|| registry_subcommand.value_of("remove-crate").unwrap())
        });
        if list_crate.installed_registry().contains(&value.to_string())
            && (remove_crate_app || remove_crate_registry)
        {
            registry_crates_location.remove_crate(value)
        }

        if list_crate.installed_git().contains(&value.to_string())
            && (remove_crate_app || remove_crate_git)
        {
            git_crates_location.remove_crate(value)
        }
    }

    let force_remove_app = app.is_present("force remove");
    let force_remove_git = git_subcommand.is_present("force remove");
    let force_remove_registry = registry_subcommand.is_present("force remove");
    // Force remove all crates without reading config file
    if force_remove_app || force_remove_git || force_remove_registry {
        if force_remove_app || force_remove_registry {
            fs::remove_dir_all(dir_path.cache_dir()).unwrap();
            fs::remove_dir_all(dir_path.src_dir()).unwrap();
        }
        if force_remove_app || force_remove_git {
            fs::remove_dir_all(dir_path.checkout_dir()).unwrap();
            fs::remove_dir_all(dir_path.db_dir()).unwrap();
        }
    }

    // Remove all crates by following config file
    let all_app = app.is_present("all");
    let all_git = git_subcommand.is_present("all");
    let all_registry = registry_subcommand.is_present("all");
    if all_app || all_git || all_registry {
        if all_app || all_registry {
            for crate_name in &list_crate.installed_registry() {
                remove_registry_all(&config_file, app, crate_name, &registry_crates_location);
            }
        }
        if all_app || all_git {
            for crate_name in &list_crate.installed_git() {
                remove_git_all(&config_file, app, crate_name, &git_crates_location);
            }
        }
    }

    // Wipe certain folder all together
    if app.is_present("wipe") {
        let value = app.value_of("wipe").unwrap();
        match value {
            "git" => fs::remove_dir_all(dir_path.git_dir()).unwrap(),
            "checkouts" => fs::remove_dir_all(dir_path.checkout_dir()).unwrap(),
            "db" => fs::remove_dir_all(dir_path.db_dir()).unwrap(),
            "registry" => fs::remove_dir_all(dir_path.registry_dir()).unwrap(),
            "cache" => fs::remove_dir_all(dir_path.cache_dir()).unwrap(),
            "index" => fs::remove_dir_all(dir_path.index_dir()).unwrap(),
            "src" => fs::remove_dir_all(dir_path.src_dir()).unwrap(),
            _ => println!("Please provide one of the given value"),
        }
    }

    // Query about config file information
    if app.is_present("query") {
        let matches = app.subcommand_matches("query").unwrap();
        query_subcommand(&config_file, matches)
    }
}

fn match_size(size: PathBuf) -> f64 {
    match get_size(size) {
        Ok(size) => (size as f64) / (1024f64.powf(2.0)),
        Err(_) => 0.0,
    }
}

fn list_subcommand(list_subcommand: &ArgMatches, list_crate: &CrateList) {
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

fn orphan_clean(
    list_crate: &CrateList,
    (orphan_app, orphan_git, orphan_registry): (bool, bool, bool),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
) {
    if orphan_app || orphan_git || orphan_registry {
        if orphan_app || orphan_registry {
            let orphan_registry_crate = list_crate.orphan_registry();
            for crate_name in &orphan_registry_crate {
                registry_crates_location.remove_crate(crate_name);
            }
        }
        if orphan_app || orphan_git {
            let orphan_git_crate = list_crate.orphan_git();
            for crate_name in &orphan_git_crate {
                git_crates_location.remove_crate(crate_name);
            }
        }
    }
}

fn query_size(
    dir_path: &DirPath,
    list_crate: &CrateList,
    (query_size_app, query_size_git, query_size_registry): (bool, bool, bool),
) {
    if query_size_app || query_size_git || query_size_registry {
        let size_git = match_size(dir_path.git_dir());
        let size_checkout = match_size(dir_path.checkout_dir());
        let size_db = match_size(dir_path.db_dir());
        let size_registry = match_size(dir_path.registry_dir());
        let size_cache = match_size(dir_path.cache_dir());
        let size_index = match_size(dir_path.index_dir());
        let size_src = match_size(dir_path.src_dir());
        if query_size_app || query_size_git {
            println!(
                "{:50} {:10.3} MB",
                format!(
                    "Total Size of .cargo/git {} crates:",
                    list_crate.installed_git().len()
                ),
                size_git
            );
            println!(
                "{:50} {:10.3} MB",
                "   |-- Size of .cargo/git/checkout folder", size_checkout
            );
            println!(
                "{:50} {:10.3} MB",
                "   |-- Size of .cargo/git/db folder", size_db
            );
        }
        if query_size_app || query_size_registry {
            println!(
                "{:50} {:10.3} MB",
                format!(
                    "Total Size of .cargo/registry {} crates:",
                    list_crate.installed_registry().len()
                ),
                size_registry
            );
            println!(
                "{:50} {:10.3} MB",
                "   |-- Size of .cargo/registry/cache folder", size_cache
            );
            println!(
                "{:50} {:10.3} MB",
                "   |-- Size of .cargo/registry/index folder", size_index
            );
            println!(
                "{:50} {:10.3} MB",
                "   |-- Size of .cargo/registry/src folder", size_src
            );
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

fn remove_git_all(
    config_file: &ConfigFile,
    app: &ArgMatches,
    crate_name: &str,
    git_crates_location: &GitDir,
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
        git_crates_location.remove_crate(crate_name);
    }
    if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
        git_crates_location.remove_crate(crate_name);
    }
}
