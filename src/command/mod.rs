use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::config_file::ConfigFile;
use crate::crate_detail::CrateDetail;
use crate::dir_path;
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

#[derive(Debug, StructOpt)]
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

#[derive(Debug, StructOpt)]
#[structopt(name= env!("CARGO_PKG_NAME"), settings=&[
    AppSettings::GlobalVersion,
    AppSettings::ArgRequiredElseHelp],
    author=env!("CARGO_PKG_AUTHORS"),
    about=env!("CARGO_PKG_DESCRIPTION")
)]
pub(crate) struct Command {
    #[structopt(long = "all", short = "a", help = "Clean up all registry & git crates")]
    all: bool,
    #[structopt(
        long = "directory",
        short = "d",
        help = "Extra list of directory of Rust projects for current command",
        env = "TRIM_DIRECTORY",
        hidden = true
    )]
    directory: Option<Vec<String>>,
    #[structopt(
        long = "dry-run",
        short = "n",
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[structopt(
        long="gc",
        short="g",
        help="Git compress to reduce size of .cargo",
        possible_values=&["all", "index", "git", "git-checkout", "git-db"]
    )]
    git_compress: Option<String>,
    #[structopt(
        long = "ignore",
        short = "i",
        help = "Extra list of ignore file name which should be ignored for current command",
        env = "TRIM_IGNORE",
        hidden = true
    )]
    ignore: Option<Vec<String>>,
    #[structopt(
        long = "light",
        short = "l",
        help = "Light cleanup repo by removing git checkout and registry source but stores git db \
                and registry archive for future compilation without internet requirement"
    )]
    light_cleanup: bool,
    #[structopt(long = "old", short = "o", help = "Clean old cache crates")]
    old: bool,
    #[structopt(
        long = "old-orphan",
        short = "z",
        help = "Clean crates which is both old and orphan"
    )]
    old_orphan: bool,
    #[structopt(
        long = "orphan",
        short = "x",
        help = "Clean orphan cache crates i.e all crates which are not present in lock file \
                generated till now use cargo trim -u to guarantee your all project generate lock \
                file"
    )]
    orphan: bool,
    #[structopt(
        long = "query",
        short = "q",
        help = "Return size of different .cargo/cache folders"
    )]
    query: bool,
    #[structopt(
        long = "remove",
        short = "r",
        help = "Remove provided crates from registry or git",
        value_name = "crate"
    )]
    remove: Option<Vec<String>>,
    #[structopt(
        long = "scan-hidden-folder",
        help = " Whether to scan hidden folder for current command",
        possible_values = &["true", "false"],
        env = "TRIM_SCAN_HIDDEN_FOLDER",
        hidden = true
    )]
    scan_hidden_folder: Option<String>,
    #[structopt(
        long = "scan-target-folder",
        help = "Whether to scan target folder for current command",
        possible_values = &["true", "false"],
        env = "TRIM_SCAN_TARGET_FOLDER",
        hidden = true
    )]
    scan_target_folder: Option<String>,
    #[structopt(
        long = "top",
        short = "t",
        help = "Show certain number of top crates which have highest size",
        value_name = "number"
    )]
    top: Option<usize>,
    #[structopt(
        long = "update",
        short = "u",
        help = "Generate and Update Cargo.lock file present inside config directory folder path"
    )]
    update: bool,
    #[structopt(long="wipe", short="w", help="Wipe folder", possible_values=&[
        "git",
        "checkouts",
        "db",
        "registry",
        "cache",
        "index",
        "index-cache",
        "src",
    ], value_name="folder")]
    wipe: Option<Vec<String>>,
    #[structopt(subcommand)]
    sub_command: Option<SubCommand>,
}

impl Command {
    pub(crate) fn run(&self) -> Result<()> {
        let dry_run = self.dry_run;

        // List out all required path
        let dir_path = dir_path::DirPath::new()?;

        // Read config file data
        let mut config_file = ConfigFile::init(dir_path.config_file())?;

        // create new CrateDetail struct
        let mut crate_detail = CrateDetail::default();

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
        if let Some(scan_hidden_folder) = &self.scan_hidden_folder {
            match scan_hidden_folder.as_str() {
                "true" => config_file.set_scan_hidden_folder(true, dry_run, false)?,
                "false" => config_file.set_scan_hidden_folder(false, dry_run, false)?,
                _ => (),
            }
        }
        if let Some(scan_target_folder) = &self.scan_target_folder {
            match scan_target_folder.as_str() {
                "true" => config_file.set_scan_target_folder(true, dry_run, false)?,
                "false" => config_file.set_scan_target_folder(false, dry_run, false)?,
                _ => (),
            }
        }

        if let Some(val) = &self.git_compress {
            git_compress(
                val,
                dir_path.index_dir(),
                dir_path.checkout_dir(),
                dir_path.db_dir(),
                dry_run,
            )?;
        }
        if self.light_cleanup {
            light_cleanup(
                dir_path.checkout_dir(),
                dir_path.src_dir(),
                dir_path.index_dir(),
                dry_run,
            );
        }
        if let Some(folders) = &self.wipe {
            for folder in folders {
                wipe_directory(folder, &dir_path, dry_run);
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
            crate::git_dir::GitDir::new(dir_path.checkout_dir(), dir_path.db_dir());

        if self.old {
            old_clean(
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                dry_run,
            );
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
            );
        }

        if let Some(crates) = &self.remove {
            remove_crates(
                crates,
                &crate_list,
                &mut registry_crates_location,
                &git_crates_location,
                &crate_detail,
                dry_run,
            );
        }

        if let Some(sub_command) = &self.sub_command {
            match &sub_command {
                SubCommand::Init(init) => init.run(&mut config_file)?,
                SubCommand::Clear(clear) => clear.run(&mut config_file)?,
                SubCommand::Config(config) => config.run(&config_file, dir_path.config_file())?,
                SubCommand::List(list) => {
                    list.run(
                        &crate_detail,
                        &crate_list,
                        config_file.directory().is_empty(),
                    );
                },
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
                },
                SubCommand::Registry(registry) => {
                    registry.run(
                        &dir_path,
                        &crate_list,
                        &crate_detail,
                        &mut registry_crates_location,
                        config_file.directory().is_empty(),
                    )?;
                },
            }
        }

        Ok(())
    }
}

// Git compress git files according to provided value if option
fn git_compress(
    value: &str,
    index_dir: &Path,
    checkout_dir: &Path,
    db_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    if (value == "index" || value == "all") && index_dir.exists() {
        for entry in fs::read_dir(index_dir).context("failed to read registry index folder")? {
            let repo_path = entry?.path();
            let file_name = repo_path
                .file_name()
                .context("Failed to get a file name / folder name")?;
            if !dry_run {
                println!(
                    "{}",
                    format!("Compressing {} registry index", file_name.to_str().unwrap()).blue()
                );
            }
            run_git_compress_commands(&repo_path, dry_run);
        }
    }
    // if git is provided it git compress all git folders
    if value.contains("git") || value == "all" {
        if (value == "git" || value == "git-checkout") && checkout_dir.exists() {
            for entry in fs::read_dir(checkout_dir).context("failed to read checkout directory")? {
                let repo_path = entry?.path();
                for rev in fs::read_dir(repo_path)
                    .context("failed to read checkout directory sub directory")?
                {
                    let rev_path = rev?.path();
                    if !dry_run {
                        println!("{}", "Compressing git checkout".blue());
                    }
                    run_git_compress_commands(&rev_path, dry_run);
                }
            }
        }
        if (value == "git" || value == "git-db") && db_dir.exists() {
            for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                let repo_path = entry?.path();
                if !dry_run {
                    println!("{}", "Compressing git db".blue());
                }
                run_git_compress_commands(&repo_path, dry_run);
            }
        }
    }
    println!("{}", "Git compress task completed".blue());
    Ok(())
}

// run combination of commands which git compress a index of registry
fn run_git_compress_commands(repo_path: &Path, dry_run: bool) {
    if dry_run {
        println!("{} git compressing {:?}", "Dry run:".yellow(), repo_path);
    } else {
        // Remove history of all checkout which will help in remove dangling commits
        if let Err(e) = std::process::Command::new("git")
            .arg("reflog")
            .arg("expire")
            .arg("--expire=now")
            .arg("--all")
            .current_dir(repo_path)
            .output()
        {
            eprintln!(
                "{}",
                format!("  \u{2514} git reflog failed to execute due to error {}", e).red()
            );
        } else {
            println!("{:70}.......Step 1/3", "  \u{251c} Completed git reflog");
        }

        // pack refs of branches/tags etc into one file know as pack-refs file for
        // effective repo access
        if let Err(e) = std::process::Command::new("git")
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
                .red()
            );
        } else {
            println!(
                "{:70}.......Step 2/3",
                "  \u{251c} Packed refs and tags successfully"
            );
        }

        // cleanup unnecessary file and optimize a local repo
        if let Err(e) = std::process::Command::new("git")
            .arg("gc")
            .arg("--aggressive")
            .arg("--prune=now")
            .current_dir(repo_path)
            .output()
        {
            eprintln!(
                "{}",
                format!("  \u{2514} git gc failed to execute due to error {}", e).red()
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
fn wipe_directory(folder: &str, dir_path: &DirPath, dry_run: bool) {
    let has_failed = match folder {
        "git" => delete_folder(dir_path.git_dir(), dry_run),
        "checkouts" => delete_folder(dir_path.checkout_dir(), dry_run),
        "db" => delete_folder(dir_path.db_dir(), dry_run),
        "registry" => delete_folder(dir_path.registry_dir(), dry_run),
        "cache" => delete_folder(dir_path.cache_dir(), dry_run),
        "index" => delete_folder(dir_path.index_dir(), dry_run),
        "index-cache" => crate::utils::delete_index_cache(dir_path.index_dir(), dry_run),
        "src" => delete_folder(dir_path.src_dir(), dry_run),
        _ => Ok(()),
    }
    .is_err();
    if has_failed {
        println!("Failed to remove {:?} directory", folder);
    } else {
        println!("{} {:?} directory", "Removed".red(), folder);
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
                .output()
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
                let message = format!("Updating {}", cargo_lock.to_str().unwrap().blue());
                println!("{}", message);
                std::process::Command::new("cargo")
                    .arg("update")
                    .current_dir(location)
                    .spawn()
                    .context("Cannot run command")?
                    .wait()
                    .context("Failed to wait for child")?;
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
) {
    let (registry_sized_cleaned, total_registry_crate_removed) =
        registry::old_clean_registry(registry_crates_location, crate_list, crate_detail, dry_run);
    let (git_sized_cleaned, total_git_crate_removed) =
        git::old_clean_git(git_crates_location, crate_list, crate_detail, dry_run);
    println!(
        "{}",
        format!(
            "{} old crates removed which had occupied {:.3} MB",
            total_git_crate_removed + total_registry_crate_removed,
            git_sized_cleaned + registry_sized_cleaned
        )
        .blue()
    );
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
                            current directory as rust project directory or pass cargo trim -d \
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
    let (registry_sized_cleaned, total_registry_crate_removed) =
        registry::old_orphan_clean_registry(
            registry_crates_location,
            crate_list,
            crate_detail,
            dry_run,
        );
    let (git_sized_cleaned, total_git_crate_removed) =
        git::old_orphan_clean_git(git_crates_location, crate_list, crate_detail, dry_run);

    println!(
        "{}",
        format!(
            "{} crates which are both old and orphan crate removed which had {:.3} MB",
            total_git_crate_removed + total_registry_crate_removed,
            git_sized_cleaned + registry_sized_cleaned
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
                            trim -d <directory> for setting rust project directory";
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
    let (registry_sized_cleaned, total_registry_crate_removed) = registry::orphan_clean_registry(
        registry_crates_location,
        crate_list,
        crate_detail,
        dry_run,
    );
    let (git_sized_cleaned, total_git_crate_removed) =
        git::orphan_clean_git(git_crates_location, crate_list, crate_detail, dry_run);

    println!(
        "{}",
        format!(
            "{} orphan crates removed which had occupied {:.3} MB",
            total_git_crate_removed + total_registry_crate_removed,
            git_sized_cleaned + registry_sized_cleaned
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
) {
    let (registry_sized_cleaned, total_registry_crate_removed) =
        registry::all_clean_registry(registry_crates_location, crate_list, crate_detail, dry_run);
    let (git_sized_cleaned, total_git_crate_removed) =
        git::all_clean_git(git_crates_location, crate_list, crate_detail, dry_run);

    println!(
        "{}",
        format!(
            "Total size of  {} crates removed :- {:.3} MB",
            total_git_crate_removed + total_registry_crate_removed,
            git_sized_cleaned + registry_sized_cleaned
        )
        .blue()
    );
}

// Remove certain crates
fn remove_crates(
    crates: &[String],
    crate_list: &CrateList,
    registry_crates_location: &mut RegistryDir,
    git_crates_location: &GitDir,
    crate_detail: &CrateDetail,
    dry_run: bool,
) {
    let mut size_cleaned = 0.0;
    for crate_name in crates {
        if crate_list.installed_registry().contains(crate_name) {
            registry_crates_location.remove_crate(crate_name, dry_run);
            size_cleaned += crate_detail.find_size_registry_all(crate_name);
        }

        if crate_list.installed_git().contains(crate_name) {
            git_crates_location.remove_crate(crate_name, dry_run);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
    }
    println!(
        "{}",
        format!("Total size removed :- {:.3} MB", size_cleaned).blue()
    );
}
