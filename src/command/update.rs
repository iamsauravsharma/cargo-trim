use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Parser;

use super::DryRun;
use crate::color::Paint as _;
use crate::output::{self, info, is_dry_run, outcome, verbose};

/// Run `cargo update` in every detected project
#[derive(Debug, Parser)]
pub(crate) struct Update {
    #[command(flatten)]
    pub(super) dry_run: DryRun,
    /// Extra arguments passed to every `cargo update` after `--`
    ///
    /// For example `cargo trim update -- --verbose` or `cargo trim update --
    /// --precise 1.0.0 -p serde`
    #[arg(last = true, value_name = "args")]
    args: Vec<String>,
}

impl Update {
    pub(super) fn run(&self, cargo_lock_files: &[PathBuf]) -> Result<()> {
        for lock_file in cargo_lock_files {
            let Some(location) = lock_file.parent() else {
                anyhow::bail!(
                    "cannot get parent directory of Cargo.lock file {}",
                    lock_file.display()
                );
            };
            let location_str = location.display();
            info!("Updating project at {}", location_str.blue());
            // under dry run cargo only reports what it would update
            let args = std::iter::once("update")
                .chain(is_dry_run().then_some("--dry-run"))
                .chain(output::is_quiet().then_some("--quiet"))
                .chain(output::is_verbose().then_some("--verbose"))
                .chain(self.args.iter().map(String::as_str))
                .collect::<Vec<_>>();
            verbose!("Running `cargo {}` in {location_str}", args.join(" "));
            if !std::process::Command::new("cargo")
                .args(&args)
                .current_dir(location)
                .status()
                .context("failed to run cargo update command")?
                .success()
            {
                anyhow::bail!("Failed to update {location_str}");
            }
        }
        outcome!("{}", "Successfully updated all dependencies".blue());
        Ok(())
    }
}
