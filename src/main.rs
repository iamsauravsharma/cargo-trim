mod config_file;
mod crate_detail;
mod create_app;
mod dir_path;
mod git_dir;
mod list_crate;
mod registry_dir;
#[cfg(test)]
mod test;

use crate::{
    config_file::ConfigFile, crate_detail::CrateDetail, dir_path::DirPath, git_dir::GitDir,
    list_crate::CrateList, registry_dir::RegistryDir,
};
use clap::ArgMatches;
use colored::*;
use fs_extra::dir::get_size;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    // set all dir path
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

    // Get Location where registry crates and git crates are stored out by cargo
    let registry_crates_location =
        registry_dir::RegistryDir::new(&dir_path.cache_dir(), &dir_path.src_dir());
    let git_crates_location = git_dir::GitDir::new(&dir_path.checkout_dir(), &dir_path.db_dir());

    // create new CrateDetail struct
    let mut crate_detail = CrateDetail::new();

    // List out crate list
    let list_crate = list_crate::CrateList::create_list(
        dir_path.bin_dir().as_path(),
        Path::new(registry_crates_location.cache()),
        Path::new(registry_crates_location.src()),
        dir_path.checkout_dir().as_path(),
        dir_path.db_dir().as_path(),
        &config_file,
        &mut crate_detail,
    );

    // Perform action of removing config file with -c flag
    clear_config(&app, &dir_path);

    // Perform git compress to .cargo/index
    git_compress(
        &app,
        &dir_path.index_dir(),
        &dir_path.checkout_dir(),
        &dir_path.db_dir(),
    );

    // Perform light cleanup
    let light_cleanup_app = app.is_present("light cleanup");
    let light_cleanup_git = git_subcommand.is_present("light cleanup");
    let light_cleanup_registry = registry_subcommand.is_present("light_cleanup");
    light_cleanup(
        &dir_path.checkout_dir(),
        &dir_path.src_dir(),
        (light_cleanup_app, light_cleanup_git, light_cleanup_registry),
    );

    // Perform action on list subcommand
    list_subcommand(app, &list_crate, &crate_detail);

    // Perform action for -q flag
    let query_size_app = app.is_present("query size");
    let query_size_git = git_subcommand.is_present("query size");
    let query_size_registry = registry_subcommand.is_present("query size");
    query_size(
        &dir_path,
        (query_size_app, query_size_git, query_size_registry),
        &list_crate,
        &crate_detail,
    );

    // Query about config file information on -s flag
    query_subcommand(&app, &config_file);

    // Perform action on -o flagmatches which remove all old crates
    let old_app = app.is_present("old clean");
    let old_registry = registry_subcommand.is_present("old clean");
    let old_git = git_subcommand.is_present("old clean");
    old_clean(
        &list_crate,
        (old_app, old_registry, old_git),
        &registry_crates_location,
        &git_crates_location,
        &crate_detail,
    );

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file on -x flag
    let orphan_app = app.is_present("orphan clean");
    let orphan_git = git_subcommand.is_present("orphan clean");
    let orphan_registry = registry_subcommand.is_present("orphan clean");
    orphan_clean(
        &list_crate,
        (orphan_app, orphan_git, orphan_registry),
        &registry_crates_location,
        &git_crates_location,
        &crate_detail,
    );

    // Remove certain crate provided with -r flag
    remove_crate(
        &list_crate,
        (&app, &git_subcommand, &registry_subcommand),
        &registry_crates_location,
        &git_crates_location,
        &crate_detail,
    );

    // Force remove all crates without reading config file
    let force_remove_app = app.is_present("force remove");
    let force_remove_git = git_subcommand.is_present("force remove");
    let force_remove_registry = registry_subcommand.is_present("force remove");
    force_remove(
        &dir_path,
        (force_remove_app, force_remove_git, force_remove_registry),
    );

    // Remove all crates by following config file
    let all_app = app.is_present("all");
    let all_git = git_subcommand.is_present("all");
    let all_registry = registry_subcommand.is_present("all");
    remove_all(
        &list_crate,
        &config_file,
        app,
        &registry_crates_location,
        &git_crates_location,
        (all_app, all_git, all_registry),
        &crate_detail,
    );

    // Show top crates
    top_crates(&app, &git_subcommand, &registry_subcommand, &crate_detail);

    let crago_toml_location = list_crate.cargo_toml_location().location_path();
    update_cargo_toml(&app, crago_toml_location);

    // Wipe certain folder all together
    wipe_directory(&app, &dir_path);
}

// Return a size of directory if present otherwise return 0.0 as a size in MB
fn folder_size(path: &PathBuf) -> f64 {
    match get_size(path) {
        Ok(size) => (size as f64) / (1024_f64.powf(2.0)),
        Err(_) => 0.0,
    }
}

// Clear Config file data
fn clear_config(app: &ArgMatches, dir_path: &DirPath) {
    if app.is_present("clear config") {
        fs::remove_file(dir_path.config_dir().as_path()).unwrap();
        println!("Cleared Config file");
    }
}

// Git compress git files
fn git_compress(app: &ArgMatches, index_dir: &PathBuf, checkout_dir: &PathBuf, db_dir: &PathBuf) {
    if app.is_present("git compress") {
        let value = app.value_of("git compress").unwrap();
        if (value == "index" || value == "all") && index_dir.exists() {
            for entry in fs::read_dir(index_dir).unwrap() {
                let repo_path = entry.unwrap().path();
                let path = repo_path.to_str().unwrap();
                if path.contains("github.com") {
                    run_git_compress_commands(&repo_path);
                }
            }
        }
        if value.contains("git") || value == "all" {
            if (value == "git" || value == "git-checkout") && checkout_dir.exists() {
                for entry in fs::read_dir(checkout_dir).unwrap() {
                    let repo_path = entry.unwrap().path();
                    for rev in fs::read_dir(repo_path).unwrap() {
                        let rev_path = rev.unwrap().path();
                        run_git_compress_commands(&rev_path)
                    }
                }
            }
            if (value == "git" || value == "git-db") && db_dir.exists() {
                for entry in fs::read_dir(db_dir).unwrap() {
                    let repo_path = entry.unwrap().path();
                    run_git_compress_commands(&repo_path);
                }
            }
        }
        println!("{}", "Git compress task completed".bright_blue());
    }
}

// run combination of commands which git compress a index of registry
fn run_git_compress_commands(repo_path: &PathBuf) {
    // Remove history of all checkout which will help in remove dangling commits
    if let Err(e) = Command::new("git")
        .arg("reflog")
        .arg("expire")
        .arg("--expire=now")
        .arg("--all")
        .current_dir(repo_path)
        .output()
    {
        panic!(format!("git reflog failed to execute due to error {}", e));
    }

    // pack refs of branches/tags etc into one file know as pack-refs file for
    // effective repo access
    if let Err(e) = Command::new("git")
        .arg("pack-refs")
        .arg("--all")
        .arg("--prune")
        .current_dir(repo_path)
        .output()
    {
        panic!(format!(
            "git pack-refs failed to execute due to error {}",
            e
        ));
    }

    // cleanup unneccessary file and optimize a local repo
    if let Err(e) = Command::new("git")
        .arg("gc")
        .arg("--aggressive")
        .arg("--prune=now")
        .current_dir(repo_path)
        .output()
    {
        panic!(format!("git gc failed to execute due to error {}", e));
    }
}

// light cleanup unused directory
fn light_cleanup(
    checkout_dir: &PathBuf,
    src_dir: &PathBuf,
    (light_cleanup_app, light_cleanup_git, light_cleanup_registry): (bool, bool, bool),
) {
    if light_cleanup_app || light_cleanup_git || light_cleanup_registry {
        if light_cleanup_app || light_cleanup_registry {
            delete_folder(checkout_dir);
        }
        if light_cleanup_app || light_cleanup_git {
            delete_folder(src_dir);
        }
    }
}

// Perform different operation for a list subcommand
fn list_subcommand(app: &ArgMatches, list_crate: &CrateList, crate_detail: &CrateDetail) {
    if app.is_present("list") {
        let list_subcommand = app.subcommand_matches("list").unwrap();
        if list_subcommand.is_present("old") {
            list_crate_type(
                crate_detail,
                list_crate.old_registry(),
                "REGISTRY OLD CRATE",
            );
            list_crate_type(crate_detail, list_crate.old_git(), "GIT OLD CRATE");
        } else if list_subcommand.is_present("orphan") {
            list_crate_type(
                crate_detail,
                list_crate.orphan_registry(),
                "REGISTRY ORPHAN CRATE",
            );
            list_crate_type(crate_detail, list_crate.orphan_git(), "GIT ORPHAN CRATE");
        } else if list_subcommand.is_present("used") {
            list_crate_type(
                crate_detail,
                list_crate.used_registry(),
                "REGISTRY USED CRATE",
            );
            list_crate_type(crate_detail, list_crate.used_git(), "GIT USED CRATE");
        } else {
            list_crate_type(
                crate_detail,
                list_crate.installed_registry(),
                "REGISTRY INSTALLED CRATE",
            );
            list_crate_type(
                crate_detail,
                list_crate.installed_git(),
                "GIT INSTALLED CRATE",
            );
        }
    }
}

// list ceratin crate type to terminal
fn list_crate_type(crate_detail: &CrateDetail, crate_type: &[String], title: &str) {
    show_title(title);

    let mut total_size = 0.0;
    for crates in crate_type {
        let size = crate_detail.find(crates, title);
        total_size += size;
        println!("|{:^40}|{:^10.3}|", crates, size);
    }

    show_total_count(crate_type, total_size);
}

// show title
fn show_title(title: &str) {
    print_dash("green");
    println!("|{:^40}|{:^10}|", title.bold(), "SIZE(MB)");
    print_dash("green");
}

// show total count using data and size
fn show_total_count(data: &[String], size: f64) {
    if data.is_empty() {
        println!("|{:^40}|{:^10}|", "NONE".red(), "0.000".red());
    }
    print_dash("green");
    let printing_statement = format!("Total no of crates:- {}", data.len()).bright_blue();
    let printing_size = format!("{:.3}", size).bright_blue();
    println!("|{:^40}|{:^10}|", printing_statement, printing_size);
    print_dash("green");
}

// print dash
fn print_dash(color: &str) {
    println!(
        "{}",
        "-----------------------------------------------------"
            .color(color)
            .bold()
    );
}

// Clean old crates
fn old_clean(
    list_crate: &CrateList,
    (old_app, old_registry, old_git): (bool, bool, bool),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
) {
    if old_app || old_registry || old_git {
        let mut size_cleaned = 0.0;
        if old_app || old_registry {
            size_cleaned += registry_crates_location
                .remove_crate_list(&crate_detail, list_crate.old_registry());
        }
        if old_app || old_git {
            size_cleaned +=
                git_crates_location.remove_crate_list(&crate_detail, list_crate.old_git());
        }
        println!(
            "{}",
            format!("Total size of old crates removed :- {:.3} MB", size_cleaned).bright_blue()
        );
    }
}

// Clean orphan crates
fn orphan_clean(
    list_crate: &CrateList,
    (orphan_app, orphan_git, orphan_registry): (bool, bool, bool),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
) {
    if orphan_app || orphan_git || orphan_registry {
        let mut size_cleaned = 0.0;
        if orphan_app || orphan_registry {
            size_cleaned += registry_crates_location
                .remove_crate_list(&crate_detail, list_crate.orphan_registry());
        }
        if orphan_app || orphan_git {
            size_cleaned +=
                git_crates_location.remove_crate_list(&crate_detail, list_crate.orphan_git());
        }
        println!(
            "{}",
            format!(
                "Total size of orphan crates removed :- {:.3} MB",
                size_cleaned
            )
            .bright_blue()
        );
    }
}

// query size of directory
fn query_size(
    dir_path: &DirPath,
    (query_size_app, query_size_git, query_size_registry): (bool, bool, bool),
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
) {
    if query_size_app || query_size_git || query_size_registry {
        if query_size_app {
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "Total size of {} .cargo/bin binary:",
                    crate_list.installed_bin().len()
                ),
                folder_size(dir_path.bin_dir())
            );
        }
        if query_size_app || query_size_git {
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "Total size of {} .cargo/git crates:",
                    crate_list.installed_git().len()
                ),
                folder_size(dir_path.git_dir())
            );
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "   ├ Size of {} .cargo/git/checkout folder",
                    crate_detail.git_crates_archive().len()
                ),
                folder_size(dir_path.checkout_dir())
            );
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "   └ Size of {} .cargo/git/db folder",
                    crate_detail.git_crates_source().len()
                ),
                folder_size(dir_path.db_dir())
            );
        }
        if query_size_app || query_size_registry {
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "Total size of {} .cargo/registry crates:",
                    crate_list.installed_registry().len()
                ),
                folder_size(dir_path.registry_dir())
            );
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "   ├ Size of {} .cargo/registry/cache folder",
                    crate_detail.registry_crates_archive().len()
                ),
                folder_size(dir_path.cache_dir())
            );
            println!(
                "{:50} {:10.2} MB",
                "   ├ Size of .cargo/registry/index folder",
                folder_size(dir_path.index_dir())
            );
            println!(
                "{:50} {:10.2} MB",
                format!(
                    "   └ Size of {} .cargo/git/src folder",
                    crate_detail.registry_crates_source().len()
                ),
                folder_size(dir_path.src_dir())
            );
        }
    }
}

// Perform all query subcommand call operation
fn query_subcommand(app: &ArgMatches, config_file: &ConfigFile) {
    if app.is_present("config") {
        let matches = app.subcommand_matches("config").unwrap();
        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        let read_directory = config_file.directory();
        if matches.is_present("directory") {
            for name in read_directory {
                println!("{}", name);
            }
        }
        if matches.is_present("include") {
            for name in read_include {
                println!("{}", name);
            }
        }
        if matches.is_present("exclude") {
            for name in read_exclude {
                println!("{}", name);
            }
        }
    }
}

// force remove all crates
fn force_remove(
    dir_path: &DirPath,
    (force_remove_app, force_remove_git, force_remove_registry): (bool, bool, bool),
) {
    if force_remove_app || force_remove_git || force_remove_registry {
        if force_remove_app || force_remove_registry {
            delete_folder(&dir_path.cache_dir());
            delete_folder(&dir_path.src_dir());
        }
        if force_remove_app || force_remove_git {
            delete_folder(&dir_path.checkout_dir());
            delete_folder(&dir_path.db_dir());
        }
        println!("{}", "Successfully removed all crates".red());
    }
}

// remove all crates by following config file information
fn remove_all(
    list_crate: &CrateList,
    config_file: &ConfigFile,
    app: &ArgMatches,
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
    (all_app, all_git, all_registry): (bool, bool, bool),
    crate_detail: &CrateDetail,
) {
    if all_app || all_git || all_registry {
        let mut total_size_cleaned = 0.0;
        if all_app || all_registry {
            for crate_name in list_crate.installed_registry() {
                total_size_cleaned += registry_crates_location.remove_all(
                    &config_file,
                    app,
                    crate_name,
                    crate_detail,
                );
            }
        }
        if all_app || all_git {
            for crate_name in list_crate.installed_git() {
                total_size_cleaned +=
                    git_crates_location.remove_all(&config_file, app, crate_name, crate_detail);
            }
        }
        println!(
            "{}",
            format!(
                "Total size of crates removed :- {:.3} MB",
                total_size_cleaned
            )
            .bright_blue()
        );
    }
}

// Remove certain crates
fn remove_crate(
    list_crate: &CrateList,
    (app, git_subcommand, registry_subcommand): (&ArgMatches, &ArgMatches, &ArgMatches),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
) {
    let remove_crate_app = app.is_present("remove-crate");
    let remove_crate_git = git_subcommand.is_present("remove-crate");
    let remove_crate_registry = registry_subcommand.is_present("remove-crate");
    if remove_crate_app || remove_crate_git || remove_crate_registry {
        let mut size_cleaned = 0.0;
        let value = app.value_of("remove-crate").unwrap_or_else(|| {
            git_subcommand
                .value_of("remove-crate")
                .unwrap_or_else(|| registry_subcommand.value_of("remove-crate").unwrap())
        });
        if list_crate.installed_registry().contains(&value.to_string())
            && (remove_crate_app || remove_crate_registry)
        {
            registry_crates_location.remove_crate(value);
            size_cleaned += crate_detail.find_size_registry_all(value);
        }

        if list_crate.installed_git().contains(&value.to_string())
            && (remove_crate_app || remove_crate_git)
        {
            git_crates_location.remove_crate(value);
            size_cleaned += crate_detail.find_size_git_all(value);
        }
        println!(
            "{}",
            format!("Total size removed :- {:.3} MB", size_cleaned).bright_blue()
        );
    }
}

// show out top n crates
fn top_crates(
    app: &ArgMatches,
    git_subcommand: &ArgMatches,
    registry_subcommand: &ArgMatches,
    crate_detail: &CrateDetail,
) {
    let top_app = app.is_present("top crates");
    let top_git = git_subcommand.is_present("top crates");
    let top_registry = registry_subcommand.is_present("top crates");
    if top_app || top_git || top_registry {
        let value = app.value_of("top crates").unwrap_or_else(|| {
            git_subcommand
                .value_of("top crates")
                .unwrap_or_else(|| registry_subcommand.value_of("top crates").unwrap())
        });

        let number = value.parse::<usize>().unwrap();
        if top_app {
            show_top_number_crates(crate_detail, "bin", number);
        }
        if top_app || top_git {
            show_top_number_crates(crate_detail, "git_archive", number);
            show_top_number_crates(crate_detail, "git_source", number);
        }
        if top_app || top_registry {
            show_top_number_crates(crate_detail, "registry_archive", number);
            show_top_number_crates(crate_detail, "registry_source", number);
        }
    }
}

// top_crates() help to list out top n crates
fn show_top_number_crates(crate_detail: &CrateDetail, crate_type: &str, number: usize) {
    let blank_hashmap = HashMap::new();
    let size_detail = match crate_type {
        "bin" => crate_detail.bin(),
        "git_archive" => crate_detail.git_crates_archive(),
        "git_source" => crate_detail.git_crates_source(),
        "registry_archive" => crate_detail.registry_crates_archive(),
        "registry_source" => crate_detail.registry_crates_source(),
        _ => &blank_hashmap,
    };
    let mut vector = size_detail.iter().collect::<Vec<_>>();
    vector.sort_by(|a, b| (b.1).cmp(a.1));
    let title = format!("Top {} {}", number, crate_type);
    show_title(title.as_str());
    if vector.is_empty() {
        println!("|{:^40}|{:^10}|", "NONE".red(), "0.000".red());
    } else if vector.len() < number {
        for i in 0..vector.len() {
            print_index_value_crate(&vector, i);
        }
    } else {
        for i in 0..number {
            print_index_value_crate(&vector, i);
        }
    }
    print_dash("green");
}

// print crate name
fn print_index_value_crate(vector: &[(&String, &u64)], i: usize) {
    let crate_name = vector[i].0;
    let size = vector[i].1;
    let size = (*size as f64) / 1024_f64.powf(2.0);
    println!("|{:^40}|{:^10.3}|", crate_name, size);
}

// Update cargo lock before doing some actions
fn update_cargo_toml(app: &ArgMatches, cargo_toml_location: &[PathBuf]) {
    if app.is_present("update") {
        for location in cargo_toml_location {
            let mut cargo_lock = location.clone();
            cargo_lock.push("Cargo.lock");
            // helps so we may not need to generate lock file again for workspace project
            if cargo_lock.exists() {
                if let Err(e) = Command::new("cargo")
                    .arg("update")
                    .current_dir(location)
                    .output()
                {
                    panic!(format!("Failed to update Cargo.lock {}", e));
                }
            }
        }
        println!("{}", "Successfully update all Cargo.lock".bright_blue())
    }
}

// Wipe certain directory
fn wipe_directory(app: &ArgMatches, dir_path: &DirPath) {
    if app.is_present("wipe") {
        let value = app.value_of("wipe").unwrap();
        match value {
            "git" => delete_folder(&dir_path.git_dir()),
            "checkouts" => delete_folder(&dir_path.checkout_dir()),
            "db" => delete_folder(&dir_path.db_dir()),
            "registry" => delete_folder(&dir_path.registry_dir()),
            "cache" => delete_folder(&dir_path.cache_dir()),
            "index" => delete_folder(&dir_path.index_dir()),
            "src" => delete_folder(&dir_path.src_dir()),
            _ => (),
        }
    }
}

// delete folder with folder path provided
fn delete_folder(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}
