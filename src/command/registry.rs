use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::{
    confirm_orphan_clean, count_in, print_dash, print_removed, query_full_width, query_print,
    show_top_number_crates, source_name_max_width,
};
use crate::dir_path::DirPath;
use crate::installed::{CrateMetaData, Sources};
use crate::list_crate::{CrateList, Selection};
use crate::remove::{RegistryDir, delete_folder, delete_index_cache};
use crate::utils::{convert_pretty, get_size};

#[derive(Debug, Parser)]
#[command(
    about = "Operate only on registry cache",
    arg_required_else_help = true
)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct Registry {
    #[arg(
        long = "all",
        short = 'a',
        help = "Clean all registry crates and empty indexes"
    )]
    all: bool,
    #[arg(
        long = "light",
        short = 'l',
        help = "Remove registry sources and index caches but keep archives"
    )]
    light_cleanup: bool,
    #[arg(
        long = "old",
        short = 'o',
        help = "Clean old registry crates",
        long_help = "Clean old registry crates. Older versions of crates which also have a newer \
                     version"
    )]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'O',
        help = "Clean registry crates which are both old and orphan"
    )]
    old_orphan: bool,
    #[arg(
        long = "orphan",
        short = 'x',
        help = "Clean orphan registry crates",
        long_help = "Clean orphan registry crates. Crates not used by any Cargo.lock file in \
                     project directories. Without any project directory every crate is orphan"
    )]
    orphan: bool,
    #[arg(long = "query", short = 'q', help = "Show size of registry cache")]
    query: bool,
    #[arg(
        long = "top",
        short = 't',
        help = "Show given number of largest registry crates",
        value_name = "number"
    )]
    top: Option<usize>,
}

impl Registry {
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        sources: &Sources,
        registry_crates_location: &mut RegistryDir,
        directory_is_empty: bool,
        dry_run: bool,
    ) -> Result<()> {
        if self.light_cleanup {
            let light_cleanup_success =
                light_cleanup_registry(dir_path.src_dir(), dir_path.index_dir(), dry_run);
            if !light_cleanup_success {
                println!("Failed to delete some folder during light cleanup");
            }
        }
        if let Some(number) = self.top {
            let max_width = source_name_max_width(sources);
            top_crates_registry(crate_list, max_width, number);
        }
        if self.query {
            let final_size = query_size_registry(dir_path, crate_list);
            query_print("Total size", &convert_pretty(final_size));
        }

        if self.old {
            print_removed(
                "old registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::Old),
                    dry_run,
                )?,
                crate_list.registry_kept(Selection::Old),
            );
        }

        if self.old_orphan && confirm_orphan_clean(directory_is_empty, dry_run)? {
            print_removed(
                "old orphan registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::OldOrphan),
                    dry_run,
                )?,
                crate_list.registry_kept(Selection::OldOrphan),
            );
        }

        if self.orphan && confirm_orphan_clean(directory_is_empty, dry_run)? {
            print_removed(
                "orphan registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::Orphan),
                    dry_run,
                )?,
                crate_list.registry_kept(Selection::Orphan),
            );
        }

        if self.all {
            print_removed(
                "registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::All),
                    dry_run,
                )?,
                crate_list.registry_kept(Selection::All),
            );
            clear_empty_index(dir_path, crate_list, dry_run)?;
        }

        Ok(())
    }
}

// Perform light cleanup of registry and return if light clean was success or
// not
pub(super) fn light_cleanup_registry(src_dir: &Path, index_dir: &Path, dry_run: bool) -> bool {
    let mut light_cleanup_success = true;
    // delete src dir
    light_cleanup_success = delete_folder(src_dir, dry_run).is_ok() && light_cleanup_success;
    // Delete out .cache folder also
    light_cleanup_success = delete_index_cache(index_dir, dry_run).is_ok() && light_cleanup_success;
    light_cleanup_success
}

// Remove index, src and cache folder of every registry left without any crate
// once all cleanable crates are cleaned
pub(super) fn clear_empty_index(
    dir_path: &DirPath,
    crate_list: &CrateList,
    dry_run: bool,
) -> Result<()> {
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
            index_removed = delete_folder(&dir.join(index_name), dry_run).is_ok() && index_removed;
        }
        if !dry_run && index_removed {
            println!(
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

// Show top registry crates
pub(super) fn top_crates_registry(crate_list: &CrateList, first_width: usize, number: usize) {
    show_top_number_crates(
        &crate_list.registry(Selection::Installed),
        "registry",
        first_width,
        number,
    );
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

// perform clean on registry crates
pub(super) fn clean_registry(
    registry_crates_location: &mut RegistryDir,
    crate_metadata_list: &[CrateMetaData],
    dry_run: bool,
) -> Result<(u64, usize)> {
    registry_crates_location.remove_crate_list(crate_metadata_list, dry_run)
}
