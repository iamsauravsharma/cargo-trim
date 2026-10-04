use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use clap::{Parser, ValueEnum};
use owo_colors::OwoColorize as _;

use self::utils::{
    confirm_orphan_clean, print_dash, print_removed, query_full_width, query_print,
    show_top_number_crates, source_name_max_width,
};
use crate::command::git::clean_git;
use crate::command::registry::{clean_registry, clear_empty_index};
use crate::command::target::{clean_all_target, query_size_target};
use crate::config_file::ConfigFile;
use crate::dir_path::DirPath;
use crate::installed::Sources;
use crate::list_crate::{CrateList, Selection};
use crate::remove::{RegistryDir, delete_folder, delete_index_cache};
use crate::utils::{convert_pretty, get_inode_handled_size};

mod config;
mod gc;
mod git;
mod list;
mod registry;
mod set;
mod target;
mod unset;
mod utils;

#[derive(Debug, Parser)]
enum SubCommand {
    Config(config::Config),
    Set(set::Set),
    Unset(unset::Unset),
    List(list::List),
    Git(git::Git),
    Registry(registry::Registry),
    Target(target::Target),
}

#[derive(Debug, Parser)]
#[command(name= clap::crate_name!(),
    version=clap::crate_version!(),
    propagate_version=true,
    arg_required_else_help=true,
    author=clap::crate_authors!(),
    about=clap::crate_description!()
)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct Command {
    #[arg(
        long = "all",
        short = 'a',
        help = "Clean all crates, empty indexes and target folders",
        long_help = "Clean all crates, empty indexes and target folders. Removes registry and git \
                     crates allowed by filter, then registry indexes left without crates, then \
                     target folders of all projects after confirmation"
    )]
    all: bool,
    #[arg(
        long = "directory",
        short = 'd',
        help = "Add project directory for this run",
        env = "TRIM_DIRECTORY",
        value_name = "path"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "dry-run",
        short = 'n',
        help = "Show what would be done without changing anything",
        global = true
    )]
    dry_run: bool,
    #[arg(
        long = "gc",
        short = 'g',
        help = "Git compress cache repositories (needs git)",
        long_help = "Git compress cache repositories (needs git). Run git repack, pack-refs, \
                     prune-packed and reflog expire, aggressive kinds also run `git gc \
                     --aggressive`",
        value_enum,
        value_name = "kind"
    )]
    git_compress: Option<Vec<gc::GitCompress>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Ignore path while scanning for this run",
        long_help = "Ignore path while scanning for this run. A relative path matches trailing \
                     path components anywhere, an absolute path matches the full path and each \
                     component may contain `*`. A folder holding `.cargo-trim-ignore` is always \
                     skipped",
        env = "TRIM_IGNORE",
        value_name = "path"
    )]
    ignore: Option<Vec<String>>,
    #[arg(
        long = "filter",
        short = 'f',
        help = "Add crate filter for this run",
        long_help = "Add crate filter for this run. Package spec `[registry:]name[@version]`. \
                     Registry and name may contain `*`, registry matches the source folder name \
                     with or without its hash suffix and version is a semver requirement (exact \
                     for a full version) or a git revision prefix. Without any plain entry every \
                     crate may be cleaned, otherwise only matching crates are cleaned. An entry \
                     starting with `!` is never cleaned and wins over other entries",
        env = "TRIM_FILTER",
        value_name = "spec"
    )]
    filter: Option<Vec<String>>,
    #[arg(
        long = "light",
        short = 'l',
        help = "Remove sources but keep archives for offline builds",
        long_help = "Remove sources but keep archives for offline builds. Removes registry \
                     sources, index caches and git checkouts while registry archives and git db \
                     stay so projects still build without internet"
    )]
    light_cleanup: bool,
    #[arg(
        long = "old",
        short = 'o',
        help = "Clean old crates",
        long_help = "Clean old crates. Older versions of registry crates which also have a newer \
                     version, and git checkouts of revisions which are not the latest fetched one"
    )]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'O',
        help = "Clean crates which are both old and orphan"
    )]
    old_orphan: bool,
    #[arg(
        long = "orphan",
        short = 'x',
        help = "Clean orphan crates",
        long_help = "Clean orphan crates. Crates not used by any Cargo.lock file in project \
                     directories. Without any project directory every crate is orphan"
    )]
    orphan: bool,
    #[arg(
        long = "query",
        short = 'q',
        help = "Show size of cache and target folders"
    )]
    query: bool,
    #[arg(
        long = "scan-hidden-folder",
        short = 'H',
        help = "Scan hidden folders for this run",
        env = "TRIM_SCAN_HIDDEN_FOLDER",
        value_name = "bool"
    )]
    scan_hidden_folder: Option<bool>,
    #[arg(
        long = "scan-target-folder",
        short = 'T',
        help = "Scan target folders for Cargo.lock files for this run",
        env = "TRIM_SCAN_TARGET_FOLDER",
        value_name = "bool"
    )]
    scan_target_folder: Option<bool>,
    #[arg(
        long = "stale-days",
        short = 's',
        help = "Days without change after which a project is stale for this run",
        long_help = "Days without change after which a project is stale for this run. Projects \
                     without any change for this many days are stale and their Cargo.lock files \
                     no longer mark crates as used, 0 turns it off",
        env = "TRIM_STALE_DAYS",
        value_name = "days"
    )]
    stale_days: Option<u32>,
    #[arg(
        long = "top",
        short = 't',
        help = "Show given number of largest crates",
        value_name = "number"
    )]
    top: Option<usize>,
    #[arg(
        long = "update",
        short = 'u',
        help = "Run `cargo update` in every detected project"
    )]
    update: bool,
    #[arg(
        long = "wipe",
        short = 'w',
        help = "Delete whole cache folder",
        value_enum,
        value_name = "folder"
    )]
    wipe: Option<Vec<Wipe>>,
    #[command(subcommand)]
    sub: Option<SubCommand>,
}

#[derive(Clone, ValueEnum, Debug)]
enum Wipe {
    Git,
    Checkouts,
    Db,
    Registry,
    Cache,
    Index,
    IndexCache,
    Src,
}

impl Command {
    #[expect(clippy::too_many_lines)]
    pub(crate) fn run(&self) -> Result<()> {
        let dry_run = self.dry_run;

        // List all required path
        let dir_path = DirPath::new()?;

        // Read config file data
        let mut config_file = ConfigFile::init(dir_path.config_file())?;

        // Apply CLI overrides to config before building crate lists so that
        // the current invocation uses the updated settings.
        if let Some(directories) = &self.directory {
            for directory in directories {
                config_file.add_directory(directory, dry_run, false)?;
            }
        }
        if let Some(ignores) = &self.ignore {
            for ignore in ignores {
                let ignore = ignore.trim_end_matches(std::path::MAIN_SEPARATOR);
                config_file.add_ignore(ignore, dry_run, false)?;
            }
        }
        if let Some(filters) = &self.filter {
            for filter in filters {
                config_file.add_filter(filter, dry_run, false)?;
            }
        }
        if let Some(scan_hidden_folder) = self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(scan_hidden_folder, dry_run, false)?;
        }
        if let Some(scan_target_folder) = self.scan_target_folder {
            config_file.set_scan_target_folder(scan_target_folder, dry_run, false)?;
        }
        if let Some(stale_days) = self.stale_days {
            config_file.set_stale_days(stale_days, dry_run, false)?;
        }

        let sources = Sources::new(dir_path.index_dir(), dir_path.db_dir())?;

        // List crates (uses the already-mutated config)
        let crate_list = CrateList::create_list(&dir_path, &config_file, &sources)?;

        if let Some(values) = &self.git_compress {
            for value in values {
                gc::git_compress(
                    value,
                    dir_path.index_dir(),
                    dir_path.checkout_dir(),
                    dir_path.db_dir(),
                    dry_run,
                )?;
            }
        }

        if self.light_cleanup {
            light_cleanup(
                dir_path.checkout_dir(),
                dir_path.src_dir(),
                dir_path.index_dir(),
                dry_run,
            );
        }

        if let Some(wipes) = &self.wipe {
            for wipe in wipes {
                wipe_directory(wipe, &dir_path, dry_run);
            }
        }

        if let Some(number) = self.top {
            top_crates(&crate_list, &sources, number);
        }

        if self.update {
            let cargo_lock_files = crate_list.cargo_lock_files().paths();
            run_cargo_update_command(cargo_lock_files, dry_run)?;
        }

        if self.query {
            query_size(&dir_path, &crate_list, &config_file)?;
        }

        let mut registry_crates_location = RegistryDir::new(
            dir_path.index_dir(),
            &crate_list.registry(Selection::Installed),
        )?;

        let directory_is_empty = config_file.directory().is_empty();

        if self.old {
            clean_crates(
                "old crates",
                &mut registry_crates_location,
                &crate_list,
                Selection::Old,
                dry_run,
            )?;
        }

        if self.old_orphan && confirm_orphan_clean(directory_is_empty, dry_run)? {
            clean_crates(
                "old orphan crates",
                &mut registry_crates_location,
                &crate_list,
                Selection::OldOrphan,
                dry_run,
            )?;
        }

        if self.orphan && confirm_orphan_clean(directory_is_empty, dry_run)? {
            clean_crates(
                "orphan crates",
                &mut registry_crates_location,
                &crate_list,
                Selection::Orphan,
                dry_run,
            )?;
        }

        if self.all {
            clean_crates(
                "crates",
                &mut registry_crates_location,
                &crate_list,
                Selection::All,
                dry_run,
            )?;
            clear_empty_index(&dir_path, &crate_list, dry_run)?;
            clean_all_target(&config_file, dry_run)?;
        }

        if let Some(sub_command) = &self.sub {
            match &sub_command {
                SubCommand::Config(config) => config.run(&config_file, dir_path.config_file())?,
                SubCommand::List(list) => {
                    let max_width = source_name_max_width(&sources);
                    list.run(&crate_list, max_width, directory_is_empty);
                }
                SubCommand::Set(set) => set.run(&mut config_file, dry_run)?,
                SubCommand::Unset(unset) => unset.run(&mut config_file, dry_run)?,
                SubCommand::Git(git) => {
                    git.run(
                        &dir_path,
                        &crate_list,
                        &sources,
                        directory_is_empty,
                        dry_run,
                    )?;
                }
                SubCommand::Target(target) => target.run(&config_file, dry_run)?,
                SubCommand::Registry(registry) => {
                    registry.run(
                        &dir_path,
                        &crate_list,
                        &sources,
                        &mut registry_crates_location,
                        directory_is_empty,
                        dry_run,
                    )?;
                }
            }
        }

        Ok(())
    }
}

// light cleanup registry directory
fn light_cleanup(checkout_dir: &Path, src_dir: &Path, index_dir: &Path, dry_run: bool) {
    let mut light_cleanup_success = true;
    // light cleanup registry
    light_cleanup_success =
        registry::light_cleanup_registry(src_dir, index_dir, dry_run) && light_cleanup_success;
    // light cleanup git
    light_cleanup_success = git::light_cleanup_git(checkout_dir, dry_run) && light_cleanup_success;
    if !light_cleanup_success {
        println!("failed to delete some folder during light cleanup");
    }
}

// wipe certain directory
fn wipe_directory(wipe: &Wipe, dir_path: &DirPath, dry_run: bool) {
    let has_failed = match wipe {
        Wipe::Git => delete_folder(dir_path.git_dir(), dry_run),
        Wipe::Checkouts => delete_folder(dir_path.checkout_dir(), dry_run),
        Wipe::Db => delete_folder(dir_path.db_dir(), dry_run),
        Wipe::Registry => delete_folder(dir_path.registry_dir(), dry_run),
        Wipe::Cache => delete_folder(dir_path.cache_dir(), dry_run),
        Wipe::Index => delete_folder(dir_path.index_dir(), dry_run),
        Wipe::IndexCache => delete_index_cache(dir_path.index_dir(), dry_run),
        Wipe::Src => delete_folder(dir_path.src_dir(), dry_run),
    }
    .is_err();
    if has_failed {
        println!("Failed to remove {wipe:?} directory");
    } else {
        println!("{} {wipe:?} directory", "Removed".red());
    }
}

fn run_cargo_update_command(cargo_lock_files: &[PathBuf], dry_run: bool) -> Result<()> {
    for lock_file in cargo_lock_files {
        let Some(location) = lock_file.parent() else {
            anyhow::bail!(
                "cannot get parent directory of Cargo.lock file {}",
                lock_file.display()
            );
        };
        let location_str = location.display();
        if dry_run {
            println!(
                "{} Updating project at \"{}\"",
                "Dry run:".yellow(),
                location_str
            );
            // in dry run mode we will not actually update the cargo lock file
            // but we will run cargo update command in dry run mode
            if !std::process::Command::new("cargo")
                .arg("update")
                .current_dir(location)
                .arg("--dry-run")
                .status()
                .context("failed to run cargo update command in dry run mode")?
                .success()
            {
                return Err(anyhow::anyhow!(
                    "Failed to update {location_str} in dry run mode"
                ));
            }
        } else {
            println!("Updating project at {}", location_str.blue());
            if !std::process::Command::new("cargo")
                .arg("update")
                .current_dir(location)
                .status()
                .context("failed to run cargo update command")?
                .success()
            {
                return Err(anyhow::anyhow!("Failed to update {location_str}"));
            }
        }
    }
    println!("{}", "Successfully updated all dependencies".blue());
    Ok(())
}

// show top n crates
fn top_crates(crate_list: &CrateList, sources: &Sources, number: usize) {
    let max_width = source_name_max_width(sources);
    show_top_number_crates(crate_list.bin(), "bin", max_width, number);
    registry::top_crates_registry(crate_list, max_width, number);
    git::top_crates_git(crate_list, max_width, number);
}

// query size of directory of cargo home folder provide some valuable size
// information
fn query_size(dir_path: &DirPath, crate_list: &CrateList, config_file: &ConfigFile) -> Result<()> {
    let mut final_size = 0_u64;
    let bin_dir_size =
        get_inode_handled_size(dir_path.bin_dir(), &mut HashSet::new()).unwrap_or(0_u64);
    final_size += bin_dir_size;
    query_print(
        &format!(
            "Total size of {} .cargo/bin binary:",
            crate_list.bin().len()
        ),
        &convert_pretty(bin_dir_size),
    );
    print_dash(query_full_width());
    final_size += registry::query_size_registry(dir_path, crate_list);
    final_size += git::query_size_git(dir_path, crate_list);
    query_print("Total size", &convert_pretty(final_size));
    print_dash(query_full_width());
    query_size_target(config_file)?;
    Ok(())
}

// Clean registry and git crates of selection and print total removed
fn clean_crates(
    label: &str,
    registry_crates_location: &mut RegistryDir,
    crate_list: &CrateList,
    selection: Selection,
    dry_run: bool,
) -> Result<()> {
    let (registry_size, registry_count) = clean_registry(
        registry_crates_location,
        &crate_list.registry(selection),
        dry_run,
    )?;
    let (git_size, git_count) = clean_git(&crate_list.git(selection), dry_run);
    print_removed(
        label,
        (registry_size + git_size, registry_count + git_count),
        crate_list.registry_kept(selection) + crate_list.git_kept(selection),
    );
    Ok(())
}
