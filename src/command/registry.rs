use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::{
    OLD_ORPHAN_CLEAN_WARNING, ORPHAN_CLEAN_WARNING, confirm_orphan_clean, count_in, print_dash,
    print_removed, query_full_width, query_print, show_top_number_crates, source_name_max_width,
};
use crate::dir_path::DirPath;
use crate::installed::{CrateMetaData, Sources};
use crate::list_crate::{CrateList, Selection};
use crate::remove::{RegistryDir, delete_folder, delete_index_cache};
use crate::utils::{convert_pretty, get_size};

#[derive(Debug, Parser)]
#[command(
    about = "Perform operation only to registry related cache file",
    arg_required_else_help = true
)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct Registry {
    #[arg(
        long = "all",
        short = 'a',
        help = "Clean up all registry crates along with their index"
    )]
    all: bool,
    #[arg(
        long = "light",
        short = 'l',
        help = "Light cleanup repo by removing registry source but stores registry archive for \
                future compilation"
    )]
    light_cleanup: bool,
    #[arg(long = "old", short = 'o', help = "Clean old registry cache crates")]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'O',
        help = "Clean registry crates which is both old and orphan"
    )]
    old_orphan: bool,
    #[arg(
        long = "orphan",
        short = 'x',
        help = "Clean orphan cache registry crates i.e all crates which are not present in lock \
                file generated till now use cargo trim -u to guarantee your all project generate \
                lock file"
    )]
    orphan: bool,
    #[arg(
        long = "query",
        short = 'q',
        help = "Return size of different .cargo/registry cache folders"
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
            );
        }

        if self.old_orphan
            && confirm_orphan_clean(directory_is_empty, OLD_ORPHAN_CLEAN_WARNING, dry_run)?
        {
            print_removed(
                "old orphan registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::OldOrphan),
                    dry_run,
                )?,
            );
        }

        if self.orphan && confirm_orphan_clean(directory_is_empty, ORPHAN_CLEAN_WARNING, dry_run)? {
            print_removed(
                "orphan registry crates",
                clean_registry(
                    registry_crates_location,
                    &crate_list.registry(Selection::Orphan),
                    dry_run,
                )?,
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
            );
            clear_empty_index(dir_path, dry_run)?;
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

// Remove index, src and cache folder of every registry, run once all registry
// crates are cleaned so no index has any crate left
pub(super) fn clear_empty_index(dir_path: &DirPath, dry_run: bool) -> Result<()> {
    let index_dir = dir_path.index_dir();
    if !index_dir.is_dir() {
        return Ok(());
    }
    let mut is_success = true;
    for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
        let index_path = entry?.path();
        let Some(index_name) = index_path.file_name() else {
            continue;
        };
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
        &crate_list.registry(Selection::All),
        "registry",
        first_width,
        number,
    );
}

// Query size of registry
pub(super) fn query_size_registry(dir_path: &DirPath, crate_list: &CrateList) -> u64 {
    let installed = crate_list.registry(Selection::All);
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
