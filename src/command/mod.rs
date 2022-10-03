use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use owo_colors::OwoColorize;

use crate::command::git::clean_git;
use crate::command::registry::clean_registry;
use crate::config_file::ConfigFile;
use crate::crate_detail::CrateDetail;
use crate::dir_path::DirPath;
use crate::git_dir::GitDir;
use crate::list_crate::CrateList;
use crate::registry_dir::RegistryDir;
use crate::utils::{convert_pretty, delete_folder, get_size, print_dash, query_print};

mod clear;
mod config;
mod git;
mod init;
mod list;
mod registry;
mod set;
mod unset;

#[derive(Debug, Parser)]
enum SubCommand {
    Init(init::Init),
    Clear(clear::Clear),
    Config(config::Config),
    Set(set::Set),
    Unset(unset::Unset),
    List(list::List),
    Git(git::Git),
    Registry(registry::Registry),
}

#[derive(Debug, Parser)]
#[command(name= clap::crate_name!(),
    version=clap::crate_version!(),
    propagate_version=true,
    arg_required_else_help=true,
    author=clap::crate_authors!(),
    about=clap::crate_description!()
)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Command {
    #[arg(long = "all", short = 'a', help = "Clean up all registry & git crates")]
    all: bool,
    #[arg(
        long = "directory",
        short = 'd',
        help = "Extra list of directory of Rust projects for current command",
        env = "TRIM_DIRECTORY"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "dry-run",
        short = 'n',
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[arg(
        long = "gc",
        short = 'g',
        value_enum,
        help = "Git compress to reduce size of .cargo (git command required)"
    )]
    git_compress: Option<Vec<GitCompress>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Extra list of ignore file name which should be ignored for current command",
        env = "TRIM_IGNORE"
    )]
    ignore: Option<Vec<String>>,
    #[arg(
        long = "light",
        short = 'l',
        help = "Light cleanup without removing files required for future compilation without \
                internet"
    )]
    light_cleanup: bool,
    #[arg(
        long,
        help = "Do not scan hidden folder for current command. Takes precedence over \
                scan-hidden-folder",
        env = "TRIM_NOT_SCAN_HIDDEN_FOLDER"
    )]
    no_scan_hidden_folder: bool,
    #[arg(
        long,
        help = "Do not scan target folder for current command. Takes precedence over  \
                scan-target-folder",
        env = "TRIM_NOT_SCAN_TARGET_FOLDER"
    )]
    no_scan_target_folder: bool,
    #[arg(long = "old", short = 'o', help = "Clean old cache crates")]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'z',
        help = "Clean crates which is both old and orphan"
    )]
    old_orphan: bool,
    #[arg(
        long = "orphan",
        short = 'x',
        help = "Clean orphan cache crates i.e all crates which are not present in lock file \
                generated till now"
    )]
    orphan: bool,
    #[arg(
        long = "query",
        short = 'q',
        help = "Return size of different .cargo/cache folders"
    )]
    query: bool,
    #[arg(
        long = "scan-hidden-folder",
        help = "Scan hidden folder for current command",
        env = "TRIM_SCAN_HIDDEN_FOLDER"
    )]
    scan_hidden_folder: bool,
    #[arg(
        long = "scan-target-folder",
        help = "Scan target folder for current command",
        env = "TRIM_SCAN_TARGET_FOLDER"
    )]
    scan_target_folder: bool,
    #[arg(
        long = "top",
        short = 't',
        help = "Show certain number of top crates which have highest size"
    )]
    top: Option<usize>,
    #[arg(
        long = "update",
        short = 'u',
        help = "Generate and Update Cargo.lock file present inside config directory folder path"
    )]
    update: bool,
    #[arg(long = "wipe", short = 'w', help = "Wipe folder", value_enum)]
    wipe: Option<Vec<Wipe>>,
    #[command(subcommand)]
    sub_command: Option<SubCommand>,
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
#[derive(Clone, ValueEnum, Debug)]
enum GitCompress {
    Index,
    GitCheckout,
    GitDb,
}

impl Command {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn run(&self) -> Result<()> {
        let dry_run = self.dry_run;

        // List out all required path
        let dir_path = DirPath::new()?;

        // Read config file data
        let mut config_file = ConfigFile::init(dir_path.config_file())?;

        // create new CrateDetail struct
        let mut crate_detail = CrateDetail::new(dir_path.index_dir(), dir_path.db_dir())?;

        // List out crates
        let crate_list =
            crate::list_crate::CrateList::create_list(&dir_path, &config_file, &mut crate_detail)?;

        if let Some(directories) = &self.directory {
            for directory in directories {
                config_file.add_directory(directory, dry_run, false)?;
            }
        }
        if let Some(ignore_file_names) = &self.ignore {
            for file in ignore_file_names {
                config_file.add_ignore_file_name(file, dry_run, false)?;
            }
        }

        if self.no_scan_hidden_folder {
            config_file.set_scan_hidden_folder(false, dry_run, false)?;
        } else if self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(true, dry_run, false)?;
        }
        if self.no_scan_target_folder {
            config_file.set_scan_target_folder(false, dry_run, false)?;
        } else if self.scan_target_folder {
            config_file.set_scan_target_folder(true, dry_run, false)?;
        }

        if let Some(values) = &self.git_compress {
            for value in values {
                git_compress(
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
            top_crates(&crate_detail, number);
        }

        if self.update {
            let cargo_toml_location = crate_list.cargo_toml_location().location_path();
            update_cargo_toml(cargo_toml_location, dry_run)?;
        }

        if self.query {
            query_size(&dir_path, &crate_list, &crate_detail);
        }

        let mut registry_crates_location = crate::registry_dir::RegistryDir::new(
            dir_path.cache_dir(),
            dir_path.src_dir(),
            dir_path.index_dir(),
            crate_list.installed_registry(),
        )?;

        let git_crates_location =
            crate::git_dir::GitDir::new(dir_path.checkout_dir(), dir_path.db_dir())?;

        if self.old {
            old_clean(
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                dry_run,
            )?;
        }

        if self.old_orphan {
            old_orphan_clean(
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                config_file.directory().is_empty(),
                dry_run,
            )?;
        }

        if self.orphan {
            orphan_clean(
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                config_file.directory().is_empty(),
                dry_run,
            )?;
        }

        if self.all {
            remove_all(
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                dry_run,
            )?;
        }

        if let Some(sub_command) = &self.sub_command {
            match &sub_command {
                SubCommand::Init(init) => init.run(&mut config_file)?,
                SubCommand::Clear(clear) => clear.run(&mut config_file)?,
                SubCommand::Config(config) => config.run(&config_file, dir_path.config_file())?,
                SubCommand::List(list) => {
                    list.run(&crate_list, config_file.directory().is_empty());
                }
                SubCommand::Set(set) => set.run(&mut config_file)?,
                SubCommand::Unset(unset) => unset.run(&mut config_file)?,
                SubCommand::Git(git) => {
                    git.run(
                        &dir_path,
                        &crate_list,
                        &crate_detail,
                        &git_crates_location,
                        config_file.directory().is_empty(),
                    )?;
                }
                SubCommand::Registry(registry) => {
                    registry.run(
                        &dir_path,
                        &crate_list,
                        &crate_detail,
                        &mut registry_crates_location,
                        config_file.directory().is_empty(),
                    )?;
                }
            }
        }

        Ok(())
    }
}

// Git compress git files according to provided value if option
fn git_compress(
    value: &GitCompress,
    index_dir: &Path,
    checkout_dir: &Path,
    db_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    let (do_index, do_git_checkout, do_git_db) = match value {
        GitCompress::Index if index_dir.exists() => (true, false, false),
        GitCompress::GitCheckout if checkout_dir.exists() => (false, true, false),
        GitCompress::GitDb if db_dir.exists() => (false, false, true),
        _ => (false, false, false),
    };
    if do_index {
        for entry in fs::read_dir(index_dir).context("failed to read registry index folder")? {
            let repo_path = entry?.path();
            let file_name = repo_path
                .file_name()
                .context("Failed to get a file name / folder name")?;
            if !dry_run {
                println!(
                    "{}",
                    format!(
                        "Compressing {} registry index",
                        file_name
                            .to_str()
                            .context("Failed to get compress file name")?
                    )
                    .blue()
                );
            }
            run_git_compress_commands(&repo_path, dry_run)?;
        }
    }
    if do_git_checkout {
        for entry in fs::read_dir(checkout_dir).context("failed to read checkout directory")? {
            let repo_path = entry?.path();
            for rev in fs::read_dir(repo_path)
                .context("failed to read checkout directory sub directory")?
            {
                let rev_path = rev?.path();
                if !dry_run {
                    println!("{}", "Compressing git checkout".blue());
                }
                run_git_compress_commands(&rev_path, dry_run)?;
            }
        }
    }
    if do_git_db {
        for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
            let repo_path = entry?.path();
            if !dry_run {
                println!("{}", "Compressing git db".blue());
            }
            run_git_compress_commands(&repo_path, dry_run)?;
        }
    }
    println!("{}", "Git compress task completed".blue());
    Ok(())
}

// run combination of commands which git compress a index of registry
fn run_git_compress_commands(repo_path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{} git compressing {:?}", "Dry run:".yellow(), repo_path);
    } else {
        // Remove history of all checkout which will help in remove dangling commits
        std::process::Command::new("git")
            .arg("reflog")
            .arg("expire")
            .arg("--expire=now")
            .arg("--all")
            .current_dir(repo_path)
            .status()
            .context("Failed to execute git reflog command")?;
        println!("{:70}.......Step 1/3", "  \u{251c} Completed git reflog");

        // pack refs of branches/tags etc into one file know as pack-refs file for
        // effective repo access
        std::process::Command::new("git")
            .arg("pack-refs")
            .arg("--all")
            .arg("--prune")
            .current_dir(repo_path)
            .status()
            .context("Failed to execute git pack-refs command")?;
        println!(
            "{:70}.......Step 2/3",
            "  \u{251c} Packed refs and tags successfully"
        );

        // cleanup unnecessary file and optimize a local repo
        std::process::Command::new("git")
            .arg("gc")
            .arg("--aggressive")
            .arg("--prune=now")
            .current_dir(repo_path)
            .status()
            .context("Failed to execute git gc command")?;
        println!(
            "{:70}.......Step 3/3",
            "  \u{2514} Cleaned up unnecessary files and optimize a files"
        );
    }
    Ok(())
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
        println!("Failed to delete some folder during light cleanup");
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
        Wipe::IndexCache => crate::utils::delete_index_cache(dir_path.index_dir(), dry_run),
        Wipe::Src => delete_folder(dir_path.src_dir(), dry_run),
    }
    .is_err();
    if has_failed {
        println!("Failed to remove {:?} directory", wipe);
    } else {
        println!("{} {:?} directory", "Removed".red(), wipe);
    }
}

// Update cargo toml

// Update cargo lock
fn update_cargo_toml(cargo_toml_location: &[PathBuf], dry_run: bool) -> Result<()> {
    for location in cargo_toml_location {
        let mut cargo_lock = location.clone();
        cargo_lock.push("Cargo.lock");
        // at first try generating lock file
        if !cargo_lock.exists() {
            std::process::Command::new("cargo")
                .arg("generate-lockfile")
                .current_dir(location)
                .status()
                .context("Failed to generate Cargo.lock")?;
        }
        // helps so we may not need to generate lock file again for workspace project
        if cargo_lock.exists() {
            if dry_run {
                println!(
                    "{} Updating lockfile at path {:?}",
                    "Dry run:".yellow(),
                    location
                );
            } else {
                let message = format!(
                    "Updating {}",
                    cargo_lock
                        .to_str()
                        .context("Failed to convert Cargo.lock file path to str")?
                        .blue()
                );
                println!("{}", message);
                std::process::Command::new("cargo")
                    .arg("update")
                    .current_dir(location)
                    .status()
                    .context("Failed to run cargo update command")?;
            }
        }
    }
    println!("{}", "Successfully updated all Cargo.lock".blue());
    Ok(())
}

// show top n crates
fn top_crates(crate_detail: &CrateDetail, number: usize) {
    crate::utils::show_top_number_crates(crate_detail.bin(), "bin", number);
    registry::top_crates_registry(crate_detail, number);
    git::top_crates_git(crate_detail, number);
}

// query size of directory of cargo home folder provide some valuable size
// information
fn query_size(dir_path: &DirPath, crate_list: &CrateList, crate_detail: &CrateDetail) {
    let mut final_size = 0_u64;
    let bin_dir_size = get_size(dir_path.bin_dir()).unwrap_or(0_u64);
    final_size += bin_dir_size;
    query_print(
        &format!(
            "Total size of {} .cargo/bin binary:",
            crate_list.installed_bin().len()
        ),
        &convert_pretty(bin_dir_size),
    );
    print_dash(crate::utils::query_full_width());
    final_size += registry::query_size_registry(dir_path, crate_list, crate_detail);
    final_size += git::query_size_git(dir_path, crate_list, crate_detail);
    query_print("Total size", &convert_pretty(final_size));
}

// Clean old crates
fn old_clean(
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> Result<()> {
    let (registry_sized_cleaned, total_registry_crate_removed) = clean_registry(
        registry_crates_location,
        crate_list.old_registry(),
        crate_detail,
        dry_run,
    )?;
    let (git_sized_cleaned, total_git_crate_removed) = clean_git(
        git_crates_location,
        crate_list.old_git(),
        crate_detail,
        dry_run,
    );
    println!(
        "{}",
        format!(
            "{} old crates removed which had occupied {}",
            total_git_crate_removed + total_registry_crate_removed,
            convert_pretty(git_sized_cleaned + registry_sized_cleaned)
        )
        .blue()
    );
    Ok(())
}

// Clean out crates which is both old and orphan
fn old_orphan_clean(
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    directory_is_empty: bool,
    dry_run: bool,
) -> Result<()> {
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This command will clean all old crates even if they are \
                            not orphan crates. Run command 'cargo trim init' to initialize \
                            current directory as rust project directory or pass cargo trim set -d \
                            <directory> for setting rust project directory";
        println!("{}", warning_text.yellow());
        let mut input = String::new();
        print!("Do you want to continue? (y/N) ");
        std::io::stdout()
            .flush()
            .context("failed to flush output stream")?;
        std::io::stdin()
            .read_line(&mut input)
            .context("error: unable to read user input")?;
        let input = input.trim().to_ascii_lowercase();
        // if answer is any instead of yes and y return
        if !["y", "yes"].contains(&input.as_str()) {
            return Ok(());
        }
    }
    let (registry_sized_cleaned, total_registry_crate_removed) = clean_registry(
        registry_crates_location,
        &crate_list.list_old_orphan_registry(),
        crate_detail,
        dry_run,
    )?;
    let (git_sized_cleaned, total_git_crate_removed) = clean_git(
        git_crates_location,
        &crate_list.list_old_orphan_git(),
        crate_detail,
        dry_run,
    );

    println!(
        "{}",
        format!(
            "{} crates which are both old and orphan crate removed which had {}",
            total_git_crate_removed + total_registry_crate_removed,
            convert_pretty(git_sized_cleaned + registry_sized_cleaned)
        )
        .blue()
    );
    Ok(())
}

// Clean orphan crates
fn orphan_clean(
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    directory_is_empty: bool,
    dry_run: bool,
) -> Result<()> {
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This command will clean all crates since all crates are \
                            classified as orphan crate. Run command 'cargo trim init' to \
                            initialize current directory as rust project directory or pass cargo \
                            trim set -d <directory> for setting rust project directory";
        println!("{}", warning_text.yellow());
        let mut input = String::new();
        print!("Do you want to continue? (y/N) ");
        std::io::stdout()
            .flush()
            .context("failed to flush output stream")?;
        std::io::stdin()
            .read_line(&mut input)
            .context("error: unable to read user input")?;
        let input = input.trim().to_ascii_lowercase();
        // If answer is not y or yes then return
        if !["y", "yes"].contains(&input.as_str()) {
            return Ok(());
        }
    }
    let (registry_sized_cleaned, total_registry_crate_removed) = clean_registry(
        registry_crates_location,
        crate_list.orphan_registry(),
        crate_detail,
        dry_run,
    )?;
    let (git_sized_cleaned, total_git_crate_removed) = clean_git(
        git_crates_location,
        crate_list.orphan_git(),
        crate_detail,
        dry_run,
    );

    println!(
        "{}",
        format!(
            "{} orphan crates removed which had occupied {}",
            total_git_crate_removed + total_registry_crate_removed,
            convert_pretty(git_sized_cleaned + registry_sized_cleaned)
        )
        .blue()
    );
    Ok(())
}

// remove all crates
fn remove_all(
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> Result<()> {
    let (registry_sized_cleaned, total_registry_crate_removed) = clean_registry(
        registry_crates_location,
        crate_list.installed_registry(),
        crate_detail,
        dry_run,
    )?;
    let (git_sized_cleaned, total_git_crate_removed) = clean_git(
        git_crates_location,
        crate_list.installed_git(),
        crate_detail,
        dry_run,
    );

    println!(
        "{}",
        format!(
            "Total size of  {} crates removed :- {}",
            total_git_crate_removed + total_registry_crate_removed,
            convert_pretty(git_sized_cleaned + registry_sized_cleaned)
        )
        .blue()
    );
    Ok(())
}
