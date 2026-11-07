use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use clap::{Parser, ValueEnum};

use super::DryRun;
use crate::color::Paint as _;
use crate::dir_path::DirPath;
use crate::output::{info, is_dry_run, verbose};

/// Git compress cache repositories (needs git)
///
/// Run git repack, pack-refs, prune-packed and reflog expire in every
/// repository of the given kinds, every kind when none is given
#[derive(Debug, Parser)]
pub(crate) struct Gc {
    /// Kinds of repository to compress
    #[arg(value_enum)]
    kinds: Vec<Repo>,
    /// Also run `git gc --aggressive`, slower but compresses more
    #[arg(long, short)]
    aggressive: bool,
    #[command(flatten)]
    pub(super) dry_run: DryRun,
}

/// Kind of git repository in cargo cache
#[derive(Clone, Copy, ValueEnum, Debug)]
enum Repo {
    /// registry index
    Index,
    /// git checkout
    Checkout,
    /// git db
    Db,
}

impl Gc {
    pub(super) fn run(&self, dir_path: &DirPath) -> Result<()> {
        let kinds = if self.kinds.is_empty() {
            &[Repo::Index, Repo::Checkout, Repo::Db][..]
        } else {
            &self.kinds
        };
        for kind in kinds {
            git_compress(*kind, self.aggressive, dir_path)?;
        }
        info!("{}", "Git compress task completed".blue());
        Ok(())
    }
}

// Git compress every repository of kind
fn git_compress(kind: Repo, is_aggressive: bool, dir_path: &DirPath) -> Result<()> {
    let (index_dir, checkout_dir, db_dir) = (
        dir_path.index_dir(),
        dir_path.checkout_dir(),
        dir_path.db_dir(),
    );
    match kind {
        Repo::Index => {
            if index_dir.is_dir() {
                for entry in
                    fs::read_dir(index_dir).context("failed to read registry index folder")?
                {
                    let repo_path = entry?.path();
                    if repo_path.join(".git").exists() {
                        run_git_compress_commands(&repo_path, "registry index", is_aggressive)?;
                    }
                }
            }
        }
        Repo::Checkout => {
            if checkout_dir.is_dir() {
                for entry in
                    fs::read_dir(checkout_dir).context("failed to read checkout directory")?
                {
                    let repo_path = entry?.path();
                    if repo_path.is_dir() {
                        for rev in fs::read_dir(repo_path)
                            .context("failed to read checkout directory sub directory")?
                        {
                            let rev_path = rev?.path();
                            run_git_compress_commands(&rev_path, "git checkout", is_aggressive)?;
                        }
                    }
                }
            }
        }
        Repo::Db => {
            if db_dir.is_dir() {
                for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                    let repo_path = entry?.path();
                    run_git_compress_commands(&repo_path, "git db", is_aggressive)?;
                }
            }
        }
    }
    Ok(())
}

// run combination of commands which git compress a repository of given kind
fn run_git_compress_commands(repo_path: &Path, kind: &str, is_aggressive: bool) -> Result<()> {
    info!(
        "{} {}",
        format!("Compressing {kind}").blue(),
        repo_path.display()
    );
    if !is_dry_run() {
        let mut commands = vec![
            // Pack unpacked objects in a repository
            (vec!["repack", "-a", "-d"], "Repack unpacked objects"),
            // pack refs of branches/tags etc into one file know as pack-refs
            // file for effective repo access
            (
                vec!["pack-refs", "--all", "--prune"],
                "Packed refs and tags successfully",
            ),
            // Remove extra objects that are already in pack files
            (vec!["prune-packed"], "Prune packed objects"),
            // Remove history of all checkout which will help in remove dangling
            // commits
            (
                vec![
                    "reflog",
                    "expire",
                    "--expire=now",
                    "--expire-unreachable=now",
                    "--all",
                ],
                "Prune older reflog",
            ),
        ];
        if is_aggressive {
            commands.push((
                vec!["gc", "--prune=now", "--aggressive"],
                "Prune aggressively",
            ));
        }
        let total_len = commands.len();
        for (pos, (args, message)) in commands.iter().enumerate() {
            let position = pos + 1;
            let symbol = if position == total_len {
                '\u{2514}'
            } else {
                '\u{251c}'
            };
            verbose!(
                "Running `git {}` in {}",
                args.join(" "),
                repo_path.display()
            );
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(repo_path)
                .output()
                .context(format!("failed to execute {position} command"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("git command at step {position}/{total_len} failed: {stderr}");
            }
            // git output is captured to keep progress tidy, verbose output
            // shows it
            for stream in [&output.stdout, &output.stderr] {
                for line in String::from_utf8_lossy(stream).lines() {
                    verbose!("  {line}");
                }
            }
            info!(
                "{:70}.......Step {position}/{total_len}",
                format!("  {symbol} {message}")
            );
        }
    }
    Ok(())
}
