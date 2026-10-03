use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::{confirm_continue, print_dash, query_full_width, query_print};
use crate::config_file::ConfigFile;
use crate::utils::{convert_pretty, delete_folder, get_size, modified_since};

#[derive(Debug, Parser)]
#[command(
    about = "Perform operation only to target folder of rust project",
    arg_required_else_help = true
)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct Target {
    #[arg(
        long = "all",
        short = 'a',
        help = "Clean target folder of all rust project"
    )]
    all: bool,
    #[arg(
        long = "dry-run",
        short = 'n',
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[arg(
        long = "list",
        short = 'l',
        help = "List target folder along with their size"
    )]
    list: bool,
    #[arg(
        long = "query",
        short = 'q',
        help = "Return size of target folder of rust project"
    )]
    query: bool,
    #[arg(
        long = "stale",
        short = 's',
        help = "Clean target folder of stale project only"
    )]
    stale: bool,
    #[arg(
        long = "top",
        short = 't',
        help = "Show certain number of target folder which have highest size",
        value_name = "number"
    )]
    top: Option<usize>,
}

impl Target {
    pub(super) fn run(&self, config_file: &ConfigFile, global_dry_run: bool) -> Result<()> {
        let dry_run = self.dry_run || global_dry_run;

        if self.list {
            print_target(&sized_target(config_file)?, "Total target folder");
        }

        if let Some(number) = self.top {
            let mut sized = sized_target(config_file)?;
            sized.truncate(number);
            print_target(&sized, &format!("Top {number} target folder"));
        }

        if self.query {
            query_size_target(config_file)?;
            print_dash(query_full_width());
        }

        if self.all {
            let target_dirs = collect(config_file, None)?;
            clean_target(&target_dirs, "target folder", dry_run)?;
        }

        if self.stale {
            let Some(cutoff) = config_file.stale_cutoff() else {
                println!(
                    "stale_days is 0 so no project is stale. Set it with 'cargo trim set \
                     --stale-days <days>' or pass --stale-days <days>"
                );
                return Ok(());
            };
            let target_dirs = collect(config_file, Some(cutoff))?;
            clean_target(&target_dirs, "stale target folder", dry_run)?;
        }

        Ok(())
    }
}

fn collect(config_file: &ConfigFile, only_stale_since: Option<SystemTime>) -> Result<Vec<PathBuf>> {
    let mut target_dirs = Vec::new();
    for directory in config_file.directory() {
        config_file.project_target_dirs(
            Path::new(directory),
            only_stale_since,
            &mut target_dirs,
        )?;
    }
    target_dirs.sort();
    target_dirs.dedup();
    Ok(target_dirs)
}

fn sized_target(config_file: &ConfigFile) -> Result<Vec<(PathBuf, u64, bool)>> {
    let cutoff = config_file.stale_cutoff();
    let mut sized = collect(config_file, None)?
        .into_iter()
        .map(|path| {
            let size = get_size(&path).unwrap_or(0);
            let is_stale = cutoff.is_some_and(|cutoff| !modified_since(&path, cutoff));
            (path, size, is_stale)
        })
        .collect::<Vec<_>>();
    sized.sort_by_key(|(_, size, _)| std::cmp::Reverse(*size));
    Ok(sized)
}

fn print_target(sized: &[(PathBuf, u64, bool)], title: &str) {
    let total = sized.iter().map(|(_, size, _)| size).sum::<u64>();
    println!(
        "{}",
        format!(
            "{title}: {} which occupy {}",
            sized.len(),
            convert_pretty(total)
        )
        .blue()
    );
    let max_width = sized
        .iter()
        .map(|(path, ..)| path.display().to_string().len())
        .max()
        .unwrap_or_default();
    for (path, size, is_stale) in sized {
        let stale_marker = if *is_stale { " (stale)" } else { "" };
        println!(
            "{:max_width$}  {:>12}{}",
            path.display(),
            convert_pretty(*size),
            stale_marker.yellow()
        );
    }
}

pub(super) fn query_size_target(config_file: &ConfigFile) -> Result<u64> {
    let sized = sized_target(config_file)?;
    let total = sized.iter().map(|(_, size, _)| size).sum::<u64>();
    query_print(
        &format!("Total size of {} project target folder:", sized.len()),
        &convert_pretty(total),
    );
    if config_file.stale_cutoff().is_some() {
        let stale = sized
            .iter()
            .filter(|(_, _, is_stale)| *is_stale)
            .collect::<Vec<_>>();
        query_print(
            &format!("   \u{2514} Size of {} stale target folder", stale.len()),
            &convert_pretty(stale.iter().map(|(_, size, _)| size).sum::<u64>()),
        );
    }
    Ok(total)
}

fn clean_target(target_dirs: &[PathBuf], label: &str, dry_run: bool) -> Result<()> {
    if target_dirs.is_empty() {
        println!("No {label} found");
        return Ok(());
    }
    if !dry_run {
        let warning_text = format!(
            "WARNING: {} {label}(s) will be deleted. Those projects will be compiled from scratch \
             the next time they are built",
            target_dirs.len()
        );
        if !confirm_continue(&warning_text)? {
            return Ok(());
        }
    }
    let mut sized_cleaned = 0;
    let mut removed = 0;
    for target_dir in target_dirs {
        let size = get_size(target_dir).unwrap_or(0);
        if delete_folder(target_dir, dry_run).is_ok() {
            sized_cleaned += size;
            removed += 1;
        } else {
            println!("Failed to remove {}", target_dir.display());
        }
    }
    println!(
        "{}",
        format!(
            "{removed} {label}(s) removed which had occupied {}",
            convert_pretty(sized_cleaned)
        )
        .blue()
    );
    Ok(())
}
