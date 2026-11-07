use std::ffi::OsStr;
use std::fs;

use anyhow::{Context as _, Result};
use clap::Parser;

use super::DryRun;
use super::clean::{Cache, CrateSelection, clean_crates, light_cleanup};
use super::utils::{count_in, print_dash, query_full_width, query_print};
use crate::color::Paint as _;
use crate::dir_path::DirPath;
use crate::list_crate::{CrateList, Selection};
use crate::output::info;
use crate::remove::{RegistryDir, remove_path};
use crate::utils::{convert_pretty, get_size};

/// Operate only on registry cache
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub(crate) struct Registry {
    /// Clean all registry crates and empty indexes
    #[arg(long, short)]
    all: bool,
    #[command(flatten)]
    pub(super) dry_run: DryRun,
    /// Remove registry sources and index caches but keep archives
    #[arg(long = "light", short)]
    light_cleanup: bool,
    #[command(flatten)]
    selection: CrateSelection,
}

impl Registry {
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        registry_dir: &mut RegistryDir,
        directory_is_empty: bool,
    ) -> Result<()> {
        if self.light_cleanup {
            light_cleanup(Cache::Registry, dir_path);
        }
        self.selection.clean(
            Cache::Registry,
            registry_dir,
            crate_list,
            directory_is_empty,
        )?;
        if self.all {
            clean_crates(
                Cache::Registry,
                "",
                registry_dir,
                crate_list,
                Selection::All,
            )?;
            clear_empty_index(dir_path, crate_list)?;
        }
        Ok(())
    }
}

// Remove index, src and cache folder of every registry left without any crate
// once all cleanable crates are cleaned
pub(super) fn clear_empty_index(dir_path: &DirPath, crate_list: &CrateList) -> Result<()> {
    let index_dir = dir_path.index_dir();
    if !index_dir.is_dir() {
        return Ok(());
    }
    // a registry still holding a crate kept by filter keeps its index
    let kept = crate_list.registry(Selection::Kept);
    let mut is_success = true;
    for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
        let index_path = entry?.path();
        let Some(index_name) = index_path.file_name() else {
            continue;
        };
        if kept
            .iter()
            .any(|crate_metadata| crate_metadata.source().map(OsStr::new) == Some(index_name))
        {
            continue;
        }
        let mut index_removed = true;
        for dir in [dir_path.src_dir(), dir_path.cache_dir(), index_dir] {
            index_removed = remove_path(&dir.join(index_name)).is_ok() && index_removed;
        }
        if index_removed {
            info!(
                "{} empty index {}",
                "Removed".red(),
                index_name.to_string_lossy()
            );
        }
        is_success = index_removed && is_success;
    }
    if !is_success {
        println!("Failed to remove some empty index");
    }
    Ok(())
}

// Query size of registry
pub(super) fn query_size_registry(dir_path: &DirPath, crate_list: &CrateList) -> u64 {
    let installed = crate_list.registry(Selection::Installed);
    let registry_dir_size = get_size(dir_path.registry_dir()).unwrap_or(0);
    query_print(
        &format!("Total size of {} .cargo/registry crates:", installed.len()),
        &convert_pretty(registry_dir_size),
    );
    query_print(
        &format!(
            "   \u{251c} Size of {} .cargo/registry/cache folder",
            count_in(&installed, dir_path.cache_dir())
        ),
        &convert_pretty(get_size(dir_path.cache_dir()).unwrap_or(0_u64)),
    );
    query_print(
        "   \u{251c} Size of .cargo/registry/index folder",
        &convert_pretty(get_size(dir_path.index_dir()).unwrap_or(0_u64)),
    );
    query_print(
        &format!(
            "   \u{2514} Size of {} .cargo/registry/src folder",
            count_in(&installed, dir_path.src_dir())
        ),
        &convert_pretty(get_size(dir_path.src_dir()).unwrap_or(0_u64)),
    );
    print_dash(query_full_width());
    registry_dir_size
}
