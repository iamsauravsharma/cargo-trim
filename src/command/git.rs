use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::OwoColorize;

use super::utils::{print_dash, show_top_number_crates};
use super::{query_full_width, query_print};
use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::dir_path::DirPath;
use crate::git_dir::GitDir;
use crate::list_crate::CrateList;
use crate::utils::{convert_pretty, get_size};
#[derive(Debug, Parser)]
#[command(
    about = "Perform operation only to git related cache file",
    arg_required_else_help = true
)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Git {
    #[arg(long = "all", short = 'a', help = "Clean up all git crates")]
    all: bool,
    #[arg(
        long = "dry-run",
        short = 'n',
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[arg(
        long = "light",
        short = 'l',
        help = "Light cleanup repo by removing git checkout but stores git db for future \
                compilation"
    )]
    light_cleanup: bool,
    #[arg(long = "old", short = 'o', help = "Clean old git cache crates")]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'z',
        help = "Clean git crates which is both old and orphan"
    )]
    old_orphan: bool,
    #[arg(
        long = "orphan",
        short = 'x',
        help = "Clean orphan cache git crates i.e all crates which are not present in lock file \
                generated till now use cargo trim -u to guarantee your all project generate lock \
                file"
    )]
    orphan: bool,
    #[arg(
        long = "query",
        short = 'q',
        help = "Return size of different .cargo/git cache folders"
    )]
    query: bool,
    #[arg(
        long = "top",
        short = 't',
        help = "Show certain number of top crates which have highest size",
        value_name = "number"
    )]
    top: Option<usize>,
}

impl Git {
    #[allow(clippy::too_many_lines)]
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        crate_detail: &CrateDetail,
        directory_is_empty: bool,
    ) -> Result<()> {
        let dry_run = self.dry_run;

        if self.light_cleanup {
            let light_cleanup_success = light_cleanup_git(dir_path.checkout_dir(), dry_run);
            if !light_cleanup_success {
                println!("Failed to delete some folder during light cleanup");
            }
        }

        if let Some(number) = self.top {
            let max_width = std::cmp::max(
                crate_detail
                    .source_infos()
                    .keys()
                    .map(String::len)
                    .max()
                    .unwrap_or(9),
                9,
            ) + 2;
            top_crates_git(crate_detail, max_width, number);
        }

        if self.query {
            let final_size = query_size_git(dir_path, crate_list, crate_detail);
            query_print("Total size", &convert_pretty(final_size));
        }

        if self.old {
            let (sized_cleaned, total_crate_removed) =
                clean_git(crate_list.old_git(), crate_detail, dry_run)?;
            println!(
                "{}",
                format!(
                    "{total_crate_removed} old crates removed which had occupied {}",
                    convert_pretty(sized_cleaned)
                )
                .blue()
            );
        }

        if self.old_orphan {
            if directory_is_empty {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This command will clean all old crates \
                                    even if they are not orphan crates. Run command 'cargo trim \
                                    init' to initialize current directory as rust project \
                                    directory or pass cargo trim set -d <directory> for setting \
                                    rust project directory";
                println!("{}", warning_text.yellow());
                let mut input = String::new();
                print!("Do you want to continue? (y/N) ");
                std::io::stdout()
                    .flush()
                    .context("failed to flush output stream")?;
                std::io::stdin()
                    .read_line(&mut input)
                    .context("error: unable to read user input")?;
                let trimmed_input = input.trim().to_ascii_lowercase();
                // if answer is any instead of yes and y return
                if !["y", "yes"].contains(&trimmed_input.as_str()) {
                    return Ok(());
                }
            }
            let (sized_cleaned, total_crate_removed) =
                clean_git(&crate_list.old_orphan_git(), crate_detail, dry_run)?;

            println!(
                "{}",
                format!(
                    "{total_crate_removed} crates which are both old and orphan crate removed \
                     which had occupied {}",
                    convert_pretty(sized_cleaned)
                )
                .blue()
            );
        }

        if self.orphan {
            if directory_is_empty {
                let warning_text = "WARNING: You have not initialized any directory as rust \
                                    project directory. This command will clean all crates since \
                                    all crates are classified as orphan crate. Run command 'cargo \
                                    trim init' to initialize current directory as rust project \
                                    directory or pass cargo trim set -d <directory> for setting \
                                    rust project directory";
                println!("{}", warning_text.yellow());
                let mut input = String::new();
                print!("Do you want to continue? (y/N) ");
                std::io::stdout()
                    .flush()
                    .context("failed to flush output stream")?;
                std::io::stdin()
                    .read_line(&mut input)
                    .context("error: unable to read user input")?;
                let trimmed_input = input.trim().to_ascii_lowercase();
                // If answer is not y or yes then return
                if !["y", "yes"].contains(&trimmed_input.as_str()) {
                    return Ok(());
                }
            }
            let (sized_cleaned, total_crate_removed) =
                clean_git(crate_list.orphan_git(), crate_detail, dry_run)?;

            println!(
                "{}",
                format!(
                    "{total_crate_removed} orphan crates removed which had occupied {}",
                    convert_pretty(sized_cleaned)
                )
                .blue()
            );
        }

        if self.all {
            let (sized_cleaned, total_crate_removed) =
                clean_git(crate_list.installed_git(), crate_detail, dry_run)?;
            println!(
                "{}",
                format!(
                    "Total size of {total_crate_removed} crates removed :- {}",
                    convert_pretty(sized_cleaned)
                )
                .blue()
            );
        }

        Ok(())
    }
}

// Perform light cleanup of git and return if light clean was success or not
pub(super) fn light_cleanup_git(checkout_dir: &Path, dry_run: bool) -> bool {
    // delete checkout dir
    crate::utils::delete_folder(checkout_dir, dry_run).is_ok()
}

// Show top git crates
pub(super) fn top_crates_git(crate_detail: &CrateDetail, first_width: usize, number: usize) {
    show_top_number_crates(
        crate_detail.git_crates_archive(),
        "git_archive",
        first_width,
        number,
    );
    show_top_number_crates(
        crate_detail.git_crates_source(),
        "git_source",
        first_width,
        number,
    );
}

pub(super) fn query_size_git(
    dir_path: &DirPath,
    crate_list: &CrateList,
    crate_detail: &CrateDetail,
) -> u64 {
    let git_dir_size = get_size(dir_path.git_dir()).unwrap_or(0_u64);
    query_print(
        &format!(
            "Total size of {} .cargo/git crates:",
            crate_list.installed_git().len()
        ),
        &convert_pretty(git_dir_size),
    );
    query_print(
        &format!(
            "   \u{251c} Size of {} .cargo/git/checkout folder",
            crate_detail.git_crates_archive().len()
        ),
        &convert_pretty(get_size(dir_path.checkout_dir()).unwrap_or(0_u64)),
    );
    query_print(
        &format!(
            "   \u{2514} Size of {} .cargo/git/db folder",
            crate_detail.git_crates_source().len()
        ),
        &convert_pretty(get_size(dir_path.db_dir()).unwrap_or(0_u64)),
    );
    print_dash(query_full_width());
    git_dir_size
}

// perform clean on git crates
pub(super) fn clean_git(
    crate_metadata_list: &[CrateMetaData],
    crate_detail: &CrateDetail,
    dry_run: bool,
) -> Result<(u64, usize)> {
    GitDir::remove_crate_list(crate_detail, crate_metadata_list, dry_run)
}
