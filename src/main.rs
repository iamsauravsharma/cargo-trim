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
    process::Command,
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

    // Get Location where registry crates and git crates are stored out by cargo
    let registry_crates_location =
        registry_dir::RegistryDir::new(&dir_path.cache_dir(), &dir_path.src_dir());
    let git_crates_location = git_dir::GitDir::new(&dir_path.checkout_dir(), &dir_path.db_dir());

    // List out crate list
    let list_crate = list_crate::CrateList::create_list(
        Path::new(registry_crates_location.cache()),
        Path::new(registry_crates_location.src()),
        dir_path.checkout_dir().as_path(),
        dir_path.db_dir().as_path(),
        &config_file,
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
    list_subcommand(app, &list_crate);

    // Perform action for -s flag
    let query_size_app = app.is_present("query size");
    let query_size_git = git_subcommand.is_present("query size");
    let query_size_registry = registry_subcommand.is_present("query size");
    query_size(
        &dir_path,
        &list_crate,
        (query_size_app, query_size_git, query_size_registry),
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
    );

    // Remove certain crate provided with -r flag
    remove_crate(
        &list_crate,
        (&app, &git_subcommand, &registry_subcommand),
        &registry_crates_location,
        &git_crates_location,
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
    );

    // Wipe certain folder all together
    wipe_directory(&app, &dir_path);
}

// Return a size of directory if present otherwise return 0.0 as a size in MB
fn match_size(path: PathBuf) -> f64 {
    match get_size(path) {
        Ok(size) => (size as f64) / (1024f64.powf(2.0)),
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
        if value == "index" || value == "all" {
            for entry in fs::read_dir(index_dir).unwrap() {
                let repo_path = entry.unwrap().path();
                let path = repo_path.to_str().unwrap();
                if path.contains("github.com") {
                    run_git_compress_commands(&repo_path);
                }
            }
        }
        if value.contains("git") || value == "all" {
            if value == "git" || value == "git-checkout" {
                for entry in fs::read_dir(checkout_dir).unwrap() {
                    let repo_path = entry.unwrap().path();
                    for rev in fs::read_dir(repo_path).unwrap() {
                        let rev_path = rev.unwrap().path();
                        run_git_compress_commands(&rev_path)
                    }
                }
            }
            if value == "git" || value == "git-db" {
                for entry in fs::read_dir(db_dir).unwrap() {
                    let repo_path = entry.unwrap().path();
                    run_git_compress_commands(&repo_path);
                }
            }
        }
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
fn list_subcommand(app: &ArgMatches, list_crate: &CrateList) {
    if app.is_present("list") {
        let list_subcommand = app.subcommand_matches("list").unwrap();
        if list_subcommand.is_present("old") {
            for crates in &list_crate.old_registry() {
                println!("{}", crates);
            }
            for crates in &list_crate.old_git() {
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
}

// Clean old crates
fn old_clean(
    list_crate: &CrateList,
    (old_app, old_registry, old_git): (bool, bool, bool),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
) {
    if old_app || old_registry || old_git {
        let mut cleaned_storage = 0.0;
        if old_app || old_registry {
            let old_registry_crate = list_crate.old_registry();
            for crate_name in &old_registry_crate {
                cleaned_storage += registry_crates_location.remove_crate(crate_name);
            }
        }
        if old_app || old_git {
            let old_git_crate = list_crate.old_git();
            for crate_name in &old_git_crate {
                cleaned_storage += git_crates_location.remove_crate(crate_name);
            }
        }
        println!("Total cleaned storage {:.3} MB", cleaned_storage);
    }
}

// Clean orphan crates
fn orphan_clean(
    list_crate: &CrateList,
    (orphan_app, orphan_git, orphan_registry): (bool, bool, bool),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
) {
    if orphan_app || orphan_git || orphan_registry {
        let mut cleaned_storage = 0.0;
        if orphan_app || orphan_registry {
            let orphan_registry_crate = list_crate.orphan_registry();
            for crate_name in &orphan_registry_crate {
                cleaned_storage += registry_crates_location.remove_crate(crate_name);
            }
        }
        if orphan_app || orphan_git {
            let orphan_git_crate = list_crate.orphan_git();
            for crate_name in &orphan_git_crate {
                cleaned_storage += git_crates_location.remove_crate(crate_name);
            }
        }
        println!("Total cleaned storage {:.3} MB", cleaned_storage);
    }
}

// query size of directory
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
fn query_subcommand(app: &ArgMatches, config_file: &ConfigFile) {
    if app.is_present("config") {
        let matches = app.subcommand_matches("config").unwrap();
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
) {
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
}

// Remove certain crates
fn remove_crate(
    list_crate: &CrateList,
    (app, git_subcommand, registry_subcommand): (&ArgMatches, &ArgMatches, &ArgMatches),
    registry_crates_location: &RegistryDir,
    git_crates_location: &GitDir,
) {
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
            registry_crates_location.remove_crate(value);
        }

        if list_crate.installed_git().contains(&value.to_string())
            && (remove_crate_app || remove_crate_git)
        {
            git_crates_location.remove_crate(value);
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

// Remove all crates from git folder
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

fn delete_folder(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}
