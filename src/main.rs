#![warn(unreachable_pub, anonymous_parameters, bare_trait_objects)]
#![deny(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

mod config_file;
mod crate_detail;
mod create_app;
mod dir_path;
mod git_dir;
mod list_crate;
mod registry_dir;
mod utils;

use crate::{
    config_file::ConfigFile,
    crate_detail::CrateDetail,
    dir_path::DirPath,
    git_dir::GitDir,
    list_crate::CrateList,
    registry_dir::RegistryDir,
    utils::{convert_pretty, delete_folder, get_size},
};
use clap::ArgMatches;
use colored::Colorize;
use std::{collections::HashMap, fs, io, io::Write, path::PathBuf, process::Command};

#[allow(clippy::too_many_lines)]
fn main() {
    // set all dir path
    let dir_path = DirPath::set_dir_path();
    let app = create_app::app().get_matches();
    let app = app.subcommand_matches("trim").unwrap();
    let arg_matches = ArgMatches::new();
    let git_subcommand = app.subcommand_matches("git").unwrap_or(&arg_matches);
    let registry_subcommand = app.subcommand_matches("registry").unwrap_or(&arg_matches);

    let dry_run_app = app.is_present("dry run");
    let dry_run_git = git_subcommand.is_present("dry run");
    let dry_run_registry = registry_subcommand.is_present("dry run");

    // Perform all read and write operation of config file flag and subcommand
    // operation and return config file
    let config_file = config_file::config_file(app, dir_path.config_file());

    // Query about config file information
    config_subcommand(app, &config_file, &dir_path.config_file());

    // Perform git compress
    git_compress(
        app,
        dir_path.index_dir(),
        dir_path.checkout_dir(),
        dir_path.db_dir(),
    );

    // Perform light cleanup
    let light_cleanup_app = app.is_present("light cleanup");
    let light_cleanup_git = git_subcommand.is_present("light cleanup");
    let light_cleanup_registry = registry_subcommand.is_present("light_cleanup");
    light_cleanup(
        dir_path.checkout_dir(),
        dir_path.src_dir(),
        dir_path.index_dir(),
        (light_cleanup_app, light_cleanup_git, light_cleanup_registry),
        (dry_run_app, dry_run_git, dry_run_registry),
    );

    // Wipe a certain folder all together
    wipe_directory(app, &dir_path);

    // create new CrateDetail struct
    let mut crate_detail = CrateDetail::default();

    // List out crates
    let list_crate = list_crate::CrateList::create_list(&dir_path, &config_file, &mut crate_detail);

    // Get Location where registry crates and git crates are stored out by cargo
    let mut registry_crates_location = registry_dir::RegistryDir::new(
        dir_path.cache_dir(),
        dir_path.src_dir(),
        dir_path.index_dir(),
        list_crate.installed_registry(),
        dry_run_app || dry_run_registry,
    );

    let git_crates_location = git_dir::GitDir::new(
        dir_path.checkout_dir(),
        dir_path.db_dir(),
        dry_run_app || dry_run_git,
    );

    // Perform action on list subcommand
    list_subcommand(app, &list_crate, &crate_detail, &config_file);

    // Perform action on -o flag matches which remove all old crates
    let old_app = app.is_present("old clean");
    let old_registry = registry_subcommand.is_present("old clean");
    let old_git = git_subcommand.is_present("old clean");
    old_clean(
        &list_crate,
        (old_app, old_registry, old_git),
        &mut registry_crates_location,
        &git_crates_location,
        &crate_detail,
    );

    // Perform action on -z flag matches which removes crates which is both old and
    // orphan
    let old_orphan_app = app.is_present("old-orphan-clean");
    let old_orphan_registry = registry_subcommand.is_present("old-orphan-clean");
    let old_orphan_git = git_subcommand.is_present("old-orphan-clean");
    old_orphan_clean(
        &list_crate,
        (old_orphan_app, old_orphan_registry, old_orphan_git),
        &mut registry_crates_location,
        &git_crates_location,
        &crate_detail,
        &config_file,
    );

    // Orphan clean a crates which is not present in directory stored in directory
    // value of config file on -x flag
    let orphan_app = app.is_present("orphan clean");
    let orphan_git = git_subcommand.is_present("orphan clean");
    let orphan_registry = registry_subcommand.is_present("orphan clean");
    orphan_clean(
        &list_crate,
        (orphan_app, orphan_git, orphan_registry),
        &mut registry_crates_location,
        &git_crates_location,
        &crate_detail,
        &config_file,
    );

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

    // Remove all crates by following config file
    let all_app = app.is_present("all");
    let all_git = git_subcommand.is_present("all");
    let all_registry = registry_subcommand.is_present("all");
    remove_all(
        &list_crate,
        &config_file,
        &mut registry_crates_location,
        &git_crates_location,
        (all_app, all_git, all_registry),
        &crate_detail,
    );

    // Remove certain crate provided with -r flag
    remove_crate(
        &list_crate,
        (app, git_subcommand, registry_subcommand),
        &mut registry_crates_location,
        &git_crates_location,
        &crate_detail,
    );

    // Show top crates
    top_crates(app, git_subcommand, registry_subcommand, &crate_detail);

    let cargo_toml_location = list_crate.cargo_toml_location().location_path();
    update_cargo_toml(app, cargo_toml_location);
}

// Git compress git files
fn git_compress(app: &ArgMatches, index_dir: &PathBuf, checkout_dir: &PathBuf, db_dir: &PathBuf) {
    if let Some(value) = app.value_of("git compress") {
        let dry_run = app.is_present("dry run");
        if (value == "index" || value == "all") && index_dir.exists() {
            for entry in fs::read_dir(index_dir).expect("failed to read registry index folder") {
                let repo_path = entry.unwrap().path();
                let file_name = repo_path
                    .file_name()
                    .expect("Failed to get a file name / folder name");
                if !dry_run {
                    println!(
                        "{}",
                        format!("Compressing {} registry index", file_name.to_str().unwrap())
                            .color("blue")
                    );
                }
                run_git_compress_commands(&repo_path, dry_run);
            }
        }
        if value.contains("git") || value == "all" {
            if (value == "git" || value == "git-checkout") && checkout_dir.exists() {
                for entry in fs::read_dir(checkout_dir).expect("failed to read checkout directory")
                {
                    let repo_path = entry.unwrap().path();
                    for rev in fs::read_dir(repo_path)
                        .expect("failed to read checkout directory sub directory")
                    {
                        let rev_path = rev.unwrap().path();
                        if !dry_run {
                            println!("{}", "Compressing git checkout".color("blue"));
                        }
                        run_git_compress_commands(&rev_path, dry_run)
                    }
                }
            }
            if (value == "git" || value == "git-db") && db_dir.exists() {
                for entry in fs::read_dir(db_dir).expect("failed to read db dir") {
                    let repo_path = entry.unwrap().path();
                    if !dry_run {
                        println!("{}", "Compressing git db".color("blue"));
                    }
                    run_git_compress_commands(&repo_path, dry_run);
                }
            }
        }
        println!("{}", "Git compress task completed".color("blue"));
    }
}

// run combination of commands which git compress a index of registry
fn run_git_compress_commands(repo_path: &PathBuf, dry_run: bool) {
    if dry_run {
        println!(
            "{} git compressing {:?}",
            "Dry run:".color("yellow"),
            repo_path
        );
    } else {
        // Remove history of all checkout which will help in remove dangling commits
        if let Err(e) = Command::new("git")
            .arg("reflog")
            .arg("expire")
            .arg("--expire=now")
            .arg("--all")
            .current_dir(repo_path)
            .output()
        {
            eprintln!(
                "{}",
                format!("  \u{2514} git reflog failed to execute due to error {}", e).color("red")
            );
        } else {
            println!("{:70}.......Step 1/3", "  \u{251c} Completed git reflog");
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
            eprintln!(
                "{}",
                format!(
                    "  \u{2514} git pack-refs failed to execute due to error {}",
                    e
                )
                .color("red")
            );
        } else {
            println!(
                "{:70}.......Step 2/3",
                "  \u{251c} Packed refs and tags successfully"
            );
        }

        // cleanup unnecessary file and optimize a local repo
        if let Err(e) = Command::new("git")
            .arg("gc")
            .arg("--aggressive")
            .arg("--prune=now")
            .current_dir(repo_path)
            .output()
        {
            eprintln!(
                "{}",
                format!("  \u{2514} git gc failed to execute due to error {}", e).color("red")
            );
        } else {
            println!(
                "{:70}.......Step 3/3",
                "  \u{2514} Cleaned up unnecessary files and optimize a files"
            );
        }
    }
}

// light cleanup registry directory
fn light_cleanup(
    checkout_dir: &PathBuf,
    src_dir: &PathBuf,
    index_dir: &PathBuf,
    (light_cleanup_app, light_cleanup_git, light_cleanup_registry): (bool, bool, bool),
    (dry_run_app, dry_run_git, dry_run_registry): (bool, bool, bool),
) {
    if light_cleanup_app || light_cleanup_git || light_cleanup_registry {
        if light_cleanup_app || light_cleanup_registry {
            let dry_run = dry_run_app || dry_run_registry;
            delete_folder(checkout_dir, dry_run);
            // Delete out .cache folder also
            delete_index_cache(index_dir, dry_run);
        }
        if light_cleanup_app || light_cleanup_git {
            let dry_run = dry_run_app || dry_run_git;
            delete_folder(src_dir, dry_run);
        }
    }
}

// list out crates which is both old and orphan
fn old_orphan_registry_list(list_crate: &CrateList) -> Vec<String> {
    let mut old_orphan_registry = Vec::new();
    let orphan_list = list_crate.orphan_registry();
    for crates in list_crate.old_registry() {
        if orphan_list.contains(crates) {
            old_orphan_registry.push(crates.to_string())
        }
    }
    old_orphan_registry
}

// list out git crates which is both old and orphan
fn old_orphan_git_list(list_crate: &CrateList) -> Vec<String> {
    let mut old_orphan_git = Vec::new();
    let orphan_list = list_crate.orphan_git();
    for crates in list_crate.old_git() {
        if orphan_list.contains(crates) {
            old_orphan_git.push(crates.to_string())
        }
    }
    old_orphan_git
}

// Perform different operation for a list subcommand
fn list_subcommand(
    app: &ArgMatches,
    list_crate: &CrateList,
    crate_detail: &CrateDetail,
    config_file: &ConfigFile,
) {
    if let Some(list_subcommand) = app.subcommand_matches("list") {
        if list_subcommand.is_present("old") {
            list_crate_type(
                crate_detail,
                list_crate.old_registry(),
                "REGISTRY OLD CRATE",
            );
            list_crate_type(crate_detail, list_crate.old_git(), "GIT OLD CRATE");
        }
        if list_subcommand.is_present("orphan") {
            list_crate_type(
                crate_detail,
                list_crate.orphan_registry(),
                "REGISTRY ORPHAN CRATE",
            );
            list_crate_type(crate_detail, list_crate.orphan_git(), "GIT ORPHAN CRATE");
            if config_file.directory().is_empty() {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This will list all crates as orphan crate. \
                                    Run command 'cargo trim init' to initialize current directory \
                                    as rust project directory or pass cargo trim -d <directory> \
                                    for setting rust project directory";
                println!("{}", warning_text.color("yellow"));
            }
        }
        if list_subcommand.is_present("used") {
            list_crate_type(
                crate_detail,
                list_crate.used_registry(),
                "REGISTRY USED CRATE",
            );
            list_crate_type(crate_detail, list_crate.used_git(), "GIT USED CRATE");
            if config_file.directory().is_empty() {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This will list no crates as used crate. \
                                    Run command 'cargo trim init' to initialize current directory \
                                    as rust project directory or pass cargo trim -d <directory> \
                                    for setting rust project directory";
                println!("{}", warning_text.color("yellow"));
            }
        }
        if list_subcommand.is_present("all") {
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
        if list_subcommand.is_present("old-orphan") {
            list_crate_type(
                crate_detail,
                &old_orphan_registry_list(list_crate),
                "REGISTRY OLD+ORPHAN CRATE",
            );
            list_crate_type(
                crate_detail,
                &old_orphan_git_list(list_crate),
                "GIT OLD+ORPHAN CRATE",
            );
            if config_file.directory().is_empty() {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This will list all old crates as old \
                                    orphan crates even if they are not orphan crates. Run command \
                                    'cargo trim init' to initialize current directory as rust \
                                    project directory or pass cargo trim -d <directory> for \
                                    setting rust project directory";
                println!("{}", warning_text.color("yellow"));
            }
        }
    }
}

// list certain crate type to terminal
fn list_crate_type(crate_detail: &CrateDetail, crate_type: &[String], title: &str) {
    let first_path_len = 40;
    let second_path_len = 10;
    let second_path_precision = 3;
    let dash_len = first_path_len + second_path_len + 3;
    show_title(title, first_path_len, second_path_len, dash_len);

    let mut total_size = 0.0;
    for crates in crate_type {
        let size = crate_detail.find(crates, title);
        total_size += size;
        println!(
            "|{:^first_width$}|{:^second_width$.precision$}|",
            crates,
            size,
            first_width = first_path_len,
            second_width = second_path_len,
            precision = second_path_precision
        );
    }
    show_total_count(
        crate_type,
        total_size,
        first_path_len,
        second_path_len,
        dash_len,
    );
}

// show title
fn show_title(title: &str, first_path_len: usize, second_path_len: usize, dash_len: usize) {
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|",
        title.bold(),
        "SIZE(MB)".bold(),
        first_width = first_path_len,
        second_width = second_path_len
    );
    print_dash(dash_len);
}

// show total count using data and size
fn show_total_count(
    data: &[String],
    size: f64,
    first_path_len: usize,
    second_path_len: usize,
    dash_len: usize,
) {
    if data.is_empty() {
        println!(
            "|{:^first_width$}|{:^second_width$}|",
            "NONE".color("red"),
            "0.000".color("red"),
            first_width = first_path_len,
            second_width = second_path_len,
        );
    }
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|",
        format!("Total no of crates:- {}", data.len()).color("blue"),
        format!("{:.3}", size).color("blue"),
        first_width = first_path_len,
        second_width = second_path_len,
    );
    print_dash(dash_len);
}

// print dash
fn print_dash(len: usize) {
    println!("{}", "-".repeat(len).color("green"));
}

// Clean old crates
fn old_clean(
    list_crate: &CrateList,
    (old_app, old_registry, old_git): (bool, bool, bool),
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
) {
    if old_app || old_registry || old_git {
        let mut size_cleaned = 0.0;
        if old_app || old_registry {
            size_cleaned +=
                registry_crates_location.remove_crate_list(crate_detail, list_crate.old_registry());
        }
        if old_app || old_git {
            size_cleaned +=
                git_crates_location.remove_crate_list(crate_detail, list_crate.old_git());
        }
        println!(
            "{}",
            format!("Total size of old crates removed :- {:.3} MB", size_cleaned).color("blue")
        );
    }
}

// Clean out crates which is both old and orphan
fn old_orphan_clean(
    list_crate: &CrateList,
    (old_orphan_app, old_orphan_registry, old_orphan_git): (bool, bool, bool),
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    config_file: &ConfigFile,
) {
    if old_orphan_app || old_orphan_registry || old_orphan_git {
        if config_file.directory().is_empty() {
            let warning_text = "WARNING: You have not initialized any directory as rust project \
                                directory. This command will clean all old crates even if they \
                                are not orphan crates. Run command 'cargo trim init' to \
                                initialize current directory as rust project directory or pass \
                                cargo trim -d <directory> for setting rust project directory";
            println!("{}", warning_text.color("yellow"));
            let mut input = String::new();
            print!("Do you want to continue? (y/N) ");
            let _ = io::stdout().flush();
            io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            let input = input.trim().to_ascii_lowercase();
            if vec!["n", "no", ""].contains(&input.as_str()) {
                return;
            }
        }
        let mut size_cleaned = 0.0;
        if old_orphan_app || old_orphan_registry {
            let old_orphan_registry = old_orphan_registry_list(list_crate);
            size_cleaned +=
                registry_crates_location.remove_crate_list(crate_detail, &old_orphan_registry);
        }
        if old_orphan_app || old_orphan_git {
            let old_orphan_git = old_orphan_git_list(list_crate);
            size_cleaned += git_crates_location.remove_crate_list(crate_detail, &old_orphan_git);
        }
        println!(
            "{}",
            format!(
                "Total size of crates which is both old and orphan crate removed :- {:.3} MB",
                size_cleaned
            )
            .color("blue")
        );
    }
}

// Clean orphan crates
fn orphan_clean(
    list_crate: &CrateList,
    (orphan_app, orphan_git, orphan_registry): (bool, bool, bool),
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    config_file: &ConfigFile,
) {
    if orphan_app || orphan_git || orphan_registry {
        if config_file.directory().is_empty() {
            let warning_text = "WARNING: You have not initialized any directory as rust project \
                                directory. This command will clean all crates since all crates \
                                are classified as orphan crate. Run command 'cargo trim init' to \
                                initialize current directory as rust project directory or pass \
                                cargo trim -d <directory> for setting rust project directory";
            println!("{}", warning_text.color("yellow"));
            let mut input = String::new();
            print!("Do you want to continue? (y/N) ");
            let _ = io::stdout().flush();
            io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            let input = input.trim().to_ascii_lowercase();
            if vec!["n", "no", ""].contains(&input.as_str()) {
                return;
            }
        }
        let mut size_cleaned = 0.0;
        if orphan_app || orphan_registry {
            size_cleaned += registry_crates_location
                .remove_crate_list(crate_detail, list_crate.orphan_registry());
        }
        if orphan_app || orphan_git {
            size_cleaned +=
                git_crates_location.remove_crate_list(crate_detail, list_crate.orphan_git());
        }
        println!(
            "{}",
            format!(
                "Total size of orphan crates removed :- {:.3} MB",
                size_cleaned
            )
            .color("blue")
        );
    }
}

// query size of directory
#[allow(clippy::too_many_lines)]
fn query_size(
    dir_path: &DirPath,
    (query_size_app, query_size_git, query_size_registry): (bool, bool, bool),
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
) {
    let mut final_size = 0_u64;
    if query_size_app || query_size_git || query_size_registry {
        let first_path_width = 50;
        let second_path_width = 10;
        let query_dash_len = first_path_width + second_path_width + 1;
        if query_size_app {
            let bin_dir_size = get_size(dir_path.bin_dir()).unwrap_or(0_u64);
            final_size += bin_dir_size;
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "Total size of {} .cargo/bin binary:",
                    crate_list.installed_bin().len()
                ),
                convert_pretty(bin_dir_size),
                first_width = first_path_width,
                second_width = second_path_width
            );
            print_dash(query_dash_len);
        }
        if query_size_app || query_size_git {
            let git_dir_size = get_size(dir_path.git_dir()).unwrap_or(0_u64);
            final_size += git_dir_size;
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "Total size of {} .cargo/git crates:",
                    crate_list.installed_git().len()
                ),
                convert_pretty(git_dir_size),
                first_width = first_path_width,
                second_width = second_path_width
            );
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "   \u{251c} Size of {} .cargo/git/checkout folder",
                    crate_detail.git_crates_archive().len()
                ),
                convert_pretty(get_size(dir_path.checkout_dir()).unwrap_or(0_u64)),
                first_width = first_path_width,
                second_width = second_path_width
            );
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "   \u{2514} Size of {} .cargo/git/db folder",
                    crate_detail.git_crates_source().len()
                ),
                convert_pretty(get_size(dir_path.checkout_dir()).unwrap_or(0_u64)),
                first_width = first_path_width,
                second_width = second_path_width
            );
            print_dash(query_dash_len);
        }
        if query_size_app || query_size_registry {
            let registry_dir_size =
                get_size(dir_path.registry_dir()).expect("failed to get size of registry dir");
            final_size += registry_dir_size;
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "Total size of {} .cargo/registry crates:",
                    crate_list.installed_registry().len()
                ),
                convert_pretty(registry_dir_size),
                first_width = first_path_width,
                second_width = second_path_width
            );
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "   \u{251c} Size of {} .cargo/registry/cache folder",
                    crate_detail.registry_crates_archive().len()
                ),
                convert_pretty(get_size(dir_path.cache_dir()).unwrap_or(0_u64)),
                first_width = first_path_width,
                second_width = second_path_width
            );
            println!(
                "{:first_width$} {:>second_width$}",
                "   \u{251c} Size of .cargo/registry/index folder",
                convert_pretty(get_size(dir_path.index_dir()).unwrap_or(0_u64)),
                first_width = first_path_width,
                second_width = second_path_width
            );
            println!(
                "{:first_width$} {:>second_width$}",
                format!(
                    "   \u{2514} Size of {} .cargo/registry/src folder",
                    crate_detail.registry_crates_source().len()
                ),
                convert_pretty(get_size(dir_path.src_dir()).unwrap_or(0_u64)),
                first_width = first_path_width,
                second_width = second_path_width
            );
            print_dash(query_dash_len);
        }
        println!(
            "{:first_width$} {:>second_width$}",
            format!(
                "Total size occupied by {}",
                std::env::var("CARGO_HOME").expect("No environmental variable CARGO_HOME present")
            ),
            convert_pretty(final_size),
            first_width = first_path_width,
            second_width = second_path_width
        );
    }
}

// Perform query about config file data
fn config_subcommand(app: &ArgMatches, config_file: &ConfigFile, config_file_location: &PathBuf) {
    if let Some(matches) = app.subcommand_matches("config") {
        if matches.is_present("directory") {
            let read_directory = config_file.directory();
            for name in read_directory {
                println!("{}", name);
            }
        }
        if matches.is_present("include") {
            let read_include = config_file.include();
            for name in read_include {
                println!("{}", name);
            }
        }
        if matches.is_present("exclude") {
            let read_exclude = config_file.exclude();
            for name in read_exclude {
                println!("{}", name);
            }
        }
        if matches.is_present("ignore_file_name") {
            let read_ignore_file_name = config_file.ignore_file_name();
            for name in read_ignore_file_name {
                println!("{}", name);
            }
        }
        if matches.is_present("location") {
            println!(
                "{}: {:?}",
                "Config file location".color("blue"),
                config_file_location
            );
        }
        if matches.is_present("print config") {
            let content = toml::to_string_pretty(config_file).unwrap();
            println!("{}", content);
        }
    }
}

// remove all crates by following config file information
fn remove_all(
    list_crate: &CrateList,
    config_file: &ConfigFile,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    (all_app, all_git, all_registry): (bool, bool, bool),
    crate_detail: &CrateDetail,
) {
    if all_app || all_git || all_registry {
        let mut total_size_cleaned = 0.0;
        if all_app || all_registry {
            for crate_name in list_crate.installed_registry() {
                total_size_cleaned +=
                    registry_crates_location.remove_all(config_file, crate_name, crate_detail);
            }
        }
        if all_app || all_git {
            for crate_name in list_crate.installed_git() {
                total_size_cleaned +=
                    git_crates_location.remove_all(config_file, crate_name, crate_detail);
            }
        }
        println!(
            "{}",
            format!(
                "Total size of crates removed :- {:.3} MB",
                total_size_cleaned
            )
            .color("blue")
        );
    }
}

// Remove certain crates
fn remove_crate(
    list_crate: &CrateList,
    (app, git_subcommand, registry_subcommand): (&ArgMatches, &ArgMatches, &ArgMatches),
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
) {
    let remove_crate_app = app.is_present("remove-crate");
    let remove_crate_git = git_subcommand.is_present("remove-crate");
    let remove_crate_registry = registry_subcommand.is_present("remove-crate");
    if remove_crate_app || remove_crate_git || remove_crate_registry {
        let mut size_cleaned = 0.0;
        let values = app
            .values_of("remove-crate")
            .unwrap_or_else(|| {
                git_subcommand
                    .values_of("remove-crate")
                    .unwrap_or_else(|| registry_subcommand.values_of("remove-crate").unwrap())
            })
            .collect::<Vec<&str>>();
        for value in values {
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
        }
        println!(
            "{}",
            format!("Total size removed :- {:.3} MB", size_cleaned).color("blue")
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

        let number = value
            .parse::<usize>()
            .expect("Cannot convert top n crates value in usize");
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
    let first_path_len = 40;
    let second_path_len = 10;
    let dash_len = first_path_len + second_path_len + 3;
    show_title(title.as_str(), first_path_len, second_path_len, dash_len);
    if vector.is_empty() {
        println!("|{:^40}|{:^10}|", "NONE".color("red"), "0.000".color("red"));
    } else if vector.len() < number {
        (0..vector.len()).for_each(|i| print_index_value_crate(&vector, i));
    } else {
        (0..number).for_each(|i| print_index_value_crate(&vector, i));
    }
    print_dash(53);
}

// print crate name
#[allow(clippy::cast_precision_loss)]
fn print_index_value_crate(vector: &[(&String, &u64)], i: usize) {
    let crate_name = vector[i].0;
    let size = vector[i].1;
    let size = (*size as f64) / 1000_f64.powi(2);
    println!("|{:^40}|{:^10.3}|", crate_name, size);
}

// Update cargo lock
fn update_cargo_toml(app: &ArgMatches, cargo_toml_location: &[PathBuf]) {
    if app.is_present("update") {
        for location in cargo_toml_location {
            let mut cargo_lock = location.clone();
            cargo_lock.push("Cargo.lock");
            // at first try generating lock file
            if !cargo_lock.exists() {
                if let Err(e) = Command::new("cargo")
                    .arg("generate-lockfile")
                    .current_dir(location)
                    .output()
                {
                    panic!(format!("Failed to generate Cargo.lock {}", e));
                }
            }
            // helps so we may not need to generate lock file again for workspace project
            if cargo_lock.exists() {
                if app.is_present("dry run") {
                    println!(
                        "{} Updating lockfile at path {:?}",
                        "Dry run:".color("yellow"),
                        location
                    )
                } else {
                    let message =
                        format!("Updating {}", cargo_lock.to_str().unwrap().color("blue"));
                    println!("{}", message);
                    Command::new("cargo")
                        .arg("update")
                        .current_dir(location)
                        .spawn()
                        .expect("Cannot run command")
                        .wait()
                        .expect("Failed to wait for child");
                }
            }
        }
        println!("{}", "Successfully update all Cargo.lock".color("blue"));
    }
}

// Wipe certain directory
fn wipe_directory(app: &ArgMatches, dir_path: &DirPath) {
    if let Some(values) = app.values_of("wipe") {
        let dry_run = app.is_present("dry run");
        for value in values {
            match value {
                "git" => delete_folder(dir_path.git_dir(), dry_run),
                "checkouts" => delete_folder(dir_path.checkout_dir(), dry_run),
                "db" => delete_folder(dir_path.db_dir(), dry_run),
                "registry" => delete_folder(dir_path.registry_dir(), dry_run),
                "cache" => delete_folder(dir_path.cache_dir(), dry_run),
                "index" => delete_folder(dir_path.index_dir(), dry_run),
                "index-cache" => delete_index_cache(dir_path.index_dir(), dry_run),
                "src" => delete_folder(dir_path.src_dir(), dry_run),
                _ => (),
            }
        }
    }
}

// delete index .cache file
fn delete_index_cache(index_dir: &PathBuf, dry_run: bool) {
    for entry in
        fs::read_dir(index_dir).expect("Failed to read index directory during force remove")
    {
        let registry_dir = entry.unwrap().path();
        for folder in
            fs::read_dir(registry_dir).expect("Failed to read registry directory in force remove")
        {
            let folder = folder.unwrap().path();
            let folder_name = folder.file_name().unwrap();
            if folder_name == ".cache" {
                delete_folder(&folder, dry_run);
            }
        }
    }
}
