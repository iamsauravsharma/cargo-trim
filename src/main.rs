mod config_file;
mod create_app;
mod dir_path;
mod list_crate;
mod registry_dir;
#[cfg(test)]
mod test;

use crate::{config_file::ConfigFile, dir_path::DirPath, registry_dir::RegistryDir};
use clap::ArgMatches;
use fs_extra::dir as dir_extra;
use std::{fs, path::Path};

fn main() {
    let dir_path = DirPath::set_dir_path();

    let config_dir = dir_path.config_dir();
    let _git_dir = dir_path.git_dir();
    let _checkout_dir = dir_path.checkout_dir();
    let _db_dir = dir_path.db_dir();
    let registry_dir = dir_path.registry_dir();
    let cache_dir = dir_path.cache_dir();
    let index_dir = dir_path.index_dir();
    let src_dir = dir_path.src_dir();

    let mut file = fs::File::open(config_dir.to_str().unwrap()).unwrap();
    let app = create_app::app();
    let app = app.subcommand_matches("trim").unwrap();

    // Perform all modification of config file flag and subcommand operation
    let config_file = config_file::modify_config_file(&mut file, app, &config_dir);

    let crates_location = registry_dir::RegistryDir::new(&cache_dir, &src_dir);

    // List out crate list
    let list_crate =
        list_crate::CrateList::create_list(Path::new(crates_location.src()), &config_file);
    let installed_crate = list_crate.installed();
    let old_crate = list_crate.old();
    let used_crate = list_crate.used();
    let orphan_crate = list_crate.orphan();

    // Perform action of removing config file with -c flag
    if app.is_present("clear config") {
        fs::remove_file(Path::new(config_dir.to_str().unwrap())).unwrap();
        println!("Cleared Config file");
    }

    // Perform action on list subcommand
    if app.is_present("list") {
        let list_subcommand = app.subcommand_matches("list").unwrap();
        if list_subcommand.is_present("old") {
            for crates in &old_crate {
                println!("{}", crates);
            }
        } else if list_subcommand.is_present("orphan") {
            for crates in &orphan_crate {
                println!("{}", crates);
            }
        } else if list_subcommand.is_present("used") {
            for crates in &used_crate {
                println!("{}", crates);
            }
        } else {
            for crates in &installed_crate {
                println!("{}", crates);
            }
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
            crates_location.remove_crate(crate_name);
        }
        println!("Successfully removed {:?} crates", old_crate.len());
    }

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file
    if app.is_present("orphan clean") {
        for crate_name in &orphan_crate {
            crates_location.remove_crate(crate_name);
        }
        println!("Successfully removed {:?} crates", orphan_crate.len());
    }

    // Remove certain crate provided with -r flag
    if app.is_present("remove-crate") {
        let value = app.value_of("remove-crate").unwrap();
        crates_location.remove_crate(value)
    }

    // Force remove all crates without reading config file
    if app.is_present("force remove") {
        fs::remove_dir_all(registry_dir.clone()).unwrap();
    }

    // Remove all crates by following config file
    if app.is_present("all") {
        for crate_name in &installed_crate {
            remove_all(&config_file, app, crate_name, &crates_location);
        }
    }

    // Wipe certain folder all together
    if app.is_present("wipe") {
        let value = app.value_of("wipe").unwrap();
        match value {
            "registry" => fs::remove_dir_all(registry_dir.clone()).unwrap(),
            "cache" => fs::remove_dir_all(cache_dir.clone()).unwrap(),
            "index" => fs::remove_dir_all(index_dir.clone()).unwrap(),
            "src" => fs::remove_dir_all(src_dir.clone()).unwrap(),
            _ => println!("Please provide one of the given four value registry, cache, index, src"),
        }
    }

    // Query about config file information
    if app.is_present("query") {
        let matches = app.subcommand_matches("query").unwrap();
        query_subcommand(&config_file, matches)
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

// Remove all crates from rigistry folder
fn remove_all(
    config_file: &ConfigFile,
    app: &ArgMatches,
    crate_name: &str,
    crates_location: &RegistryDir,
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
        crates_location.remove_crate(crate_name);
    }
    if !cmd_exclude.contains(crate_name) && !read_exclude.contains(crate_name) {
        crates_location.remove_crate(crate_name);
    }
}
