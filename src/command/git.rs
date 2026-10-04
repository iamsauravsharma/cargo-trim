use std::path::Path;

use anyhow::Result;
use clap::Parser;

use super::utils::{
    OLD_ORPHAN_CLEAN_WARNING, ORPHAN_CLEAN_WARNING, confirm_orphan_clean, count_in, print_dash,
    print_removed, show_top_number_crates, source_name_max_width,
};
use super::{query_full_width, query_print};
use crate::dir_path::DirPath;
use crate::installed::{CrateMetaData, Sources};
use crate::list_crate::{CrateList, Selection};
use crate::remove::{delete_folder, remove_git_crates};
use crate::utils::{convert_pretty, get_size};
#[derive(Debug, Parser)]
#[command(
    about = "Perform operation only to git related cache file",
    arg_required_else_help = true
)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct Git {
    #[arg(long = "all", short = 'a', help = "Clean up all git crates")]
    all: bool,
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
        short = 'O',
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
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        sources: &Sources,
        directory_is_empty: bool,
        dry_run: bool,
    ) -> Result<()> {
        if self.light_cleanup {
            let light_cleanup_success = light_cleanup_git(dir_path.checkout_dir(), dry_run);
            if !light_cleanup_success {
                println!("Failed to delete some folder during light cleanup");
            }
        }

        if let Some(number) = self.top {
            let max_width = source_name_max_width(sources);
            top_crates_git(crate_list, max_width, number);
        }

        if self.query {
            let final_size = query_size_git(dir_path, crate_list);
            query_print("Total size", &convert_pretty(final_size));
        }

        if self.old {
            print_removed(
                "old git crates",
                clean_git(&crate_list.git(Selection::Old), dry_run),
            );
        }

        if self.old_orphan
            && confirm_orphan_clean(directory_is_empty, OLD_ORPHAN_CLEAN_WARNING, dry_run)?
        {
            print_removed(
                "old orphan git crates",
                clean_git(&crate_list.git(Selection::OldOrphan), dry_run),
            );
        }

        if self.orphan && confirm_orphan_clean(directory_is_empty, ORPHAN_CLEAN_WARNING, dry_run)? {
            print_removed(
                "orphan git crates",
                clean_git(&crate_list.git(Selection::Orphan), dry_run),
            );
        }

        if self.all {
            print_removed(
                "git crates",
                clean_git(&crate_list.git(Selection::All), dry_run),
            );
        }

        Ok(())
    }
}

// Perform light cleanup of git and return if light clean was success or not
pub(super) fn light_cleanup_git(checkout_dir: &Path, dry_run: bool) -> bool {
    // delete checkout dir
    delete_folder(checkout_dir, dry_run).is_ok()
}

// Show top git crates
pub(super) fn top_crates_git(crate_list: &CrateList, first_width: usize, number: usize) {
    show_top_number_crates(&crate_list.git(Selection::All), "git", first_width, number);
}

pub(super) fn query_size_git(dir_path: &DirPath, crate_list: &CrateList) -> u64 {
    let installed = crate_list.git(Selection::All);
    let git_dir_size = get_size(dir_path.git_dir()).unwrap_or(0_u64);
    query_print(
        &format!("Total size of {} .cargo/git crates:", installed.len()),
        &convert_pretty(git_dir_size),
    );
    query_print(
        &format!(
            "   \u{251c} Size of {} .cargo/git/checkouts folder",
            count_in(&installed, dir_path.checkout_dir())
        ),
        &convert_pretty(get_size(dir_path.checkout_dir()).unwrap_or(0_u64)),
    );
    query_print(
        &format!(
            "   \u{2514} Size of {} .cargo/git/db folder",
            count_in(&installed, dir_path.db_dir())
        ),
        &convert_pretty(get_size(dir_path.db_dir()).unwrap_or(0_u64)),
    );
    print_dash(query_full_width());
    git_dir_size
}

// perform clean on git crates
pub(super) fn clean_git(crate_metadata_list: &[CrateMetaData], dry_run: bool) -> (u64, usize) {
    remove_git_crates(crate_metadata_list, dry_run)
}
