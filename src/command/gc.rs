use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use clap::ValueEnum;
use owo_colors::OwoColorize as _;

#[derive(Clone, ValueEnum, Debug)]
pub(super) enum GitCompress {
    AggressiveCheckout,
    AggressiveDb,
    AggressiveIndex,
    Checkout,
    Db,
    Index,
}

enum GitCompressAction {
    Index,
    Checkout,
    Db,
}

// Git compress git files according to provided value if option
pub(super) fn git_compress(
    value: &GitCompress,
    index_dir: &Path,
    checkout_dir: &Path,
    db_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    let (git_compress_action, is_aggressive) = match value {
        GitCompress::AggressiveIndex if index_dir.exists() => {
            (Some(GitCompressAction::Index), true)
        }
        GitCompress::AggressiveCheckout if checkout_dir.exists() => {
            (Some(GitCompressAction::Checkout), true)
        }
        GitCompress::AggressiveDb if db_dir.exists() => (Some(GitCompressAction::Db), true),
        GitCompress::Index if index_dir.exists() => (Some(GitCompressAction::Index), false),
        GitCompress::Checkout if checkout_dir.exists() => {
            (Some(GitCompressAction::Checkout), false)
        }
        GitCompress::Db if db_dir.exists() => (Some(GitCompressAction::Db), false),
        _ => (None, false),
    };
    if let Some(git_compress) = git_compress_action {
        match git_compress {
            GitCompressAction::Index => {
                if index_dir.exists() && index_dir.is_dir() {
                    for entry in
                        fs::read_dir(index_dir).context("failed to read registry index folder")?
                    {
                        let repo_path = entry?.path();
                        let file_name = repo_path
                            .file_name()
                            .context("failed to get a file name / folder name")?;
                        let mut git_folder = repo_path.clone();
                        git_folder.push(".git");
                        if git_folder.exists() {
                            if !dry_run {
                                println!(
                                    "{}",
                                    format!(
                                        "Compressing {} registry index",
                                        file_name
                                            .to_str()
                                            .context("failed to get compress file name")?
                                    )
                                    .blue()
                                );
                            }
                            run_git_compress_commands(&repo_path, dry_run, is_aggressive)?;
                        }
                    }
                }
            }
            GitCompressAction::Checkout => {
                if checkout_dir.is_dir() && checkout_dir.exists() {
                    for entry in
                        fs::read_dir(checkout_dir).context("failed to read checkout directory")?
                    {
                        let repo_path = entry?.path();
                        if repo_path.exists() && repo_path.is_dir() {
                            for rev in fs::read_dir(repo_path)
                                .context("failed to read checkout directory sub directory")?
                            {
                                let rev_path = rev?.path();
                                if !dry_run {
                                    println!("{}", "Compressing git checkout".blue());
                                }
                                run_git_compress_commands(&rev_path, dry_run, is_aggressive)?;
                            }
                        }
                    }
                }
            }
            GitCompressAction::Db => {
                if db_dir.exists() && db_dir.is_dir() {
                    for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                        let repo_path = entry?.path();
                        if !dry_run {
                            println!("{}", "Compressing git db".blue());
                        }
                        run_git_compress_commands(&repo_path, dry_run, is_aggressive)?;
                    }
                }
            }
        }
    }
    println!("{}", "Git compress task completed".blue());
    Ok(())
}

// run combination of commands which git compress a index of registry
fn run_git_compress_commands(repo_path: &Path, dry_run: bool, is_aggressive: bool) -> Result<()> {
    if dry_run {
        println!(
            "{} git compressing {}",
            "Dry run:".yellow(),
            repo_path.display()
        );
    } else {
        let mut commands = vec![
            // Pack unpacked objects in a repository
            (vec!["repack", "-a", "-d"], "Repack unpacked objects"),
            // pack refs of branches/tags etc into one file know as pack-refs file for
            // effective repo access
            (
                vec!["pack-refs", "--all", "--prune"],
                "Packed refs and tags successfully",
            ),
            // Remove extra objects that are already in pack files
            (vec!["prune-packed"], "Prune packed objects"),
            // Remove history of all checkout which will help in remove dangling commits
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
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(repo_path)
                .output()
                .context(format!("failed to execute {position} command"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("git command at step {position}/{total_len} failed: {stderr}");
            }
            println!(
                "{:70}.......Step {position}/{total_len}",
                format!("  {symbol} {message}")
            );
        }
    }
    Ok(())
}
