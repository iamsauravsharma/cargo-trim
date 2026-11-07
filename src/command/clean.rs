use anyhow::Result;
use clap::Args;

use super::utils::{confirm_orphan_clean, print_removed};
use crate::dir_path::DirPath;
use crate::list_crate::{CrateList, Selection};
use crate::remove::{RegistryDir, delete_folder, delete_index_cache, remove_git_crates};

/// Part of cargo cache a command operates on
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Cache {
    /// registry and git cache
    Every,
    Registry,
    Git,
}

impl Cache {
    fn has_registry(self) -> bool {
        self != Self::Git
    }

    fn has_git(self) -> bool {
        self != Self::Registry
    }

    /// name of crates of given kind in this cache such as `old git crates`
    fn crates_label(self, kind: &str) -> String {
        let cache = match self {
            Self::Every => "",
            Self::Registry => "registry",
            Self::Git => "git",
        };
        [kind, cache, "crates"]
            .into_iter()
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Options choosing which crates to clean, shared by every command cleaning
/// crates
#[derive(Debug, Args)]
pub(super) struct CrateSelection {
    /// Clean old crates
    ///
    /// Older versions of registry crates which also have a newer version, and
    /// git checkouts of revisions which are not the latest fetched one
    #[arg(long, short)]
    old: bool,
    /// Clean crates which are both old and orphan
    #[arg(long, short = 'O')]
    old_orphan: bool,
    /// Clean orphan crates
    ///
    /// Crates not used by any Cargo.lock file in project directories. Without
    /// any project directory every crate is orphan
    #[arg(long, short = 'x')]
    orphan: bool,
}

impl CrateSelection {
    /// clean crates of every chosen selection in cache, orphan selections are
    /// confirmed first when no project directory is set
    pub(super) fn clean(
        &self,
        cache: Cache,
        registry_dir: &mut RegistryDir,
        crate_list: &CrateList,
        directory_is_empty: bool,
    ) -> Result<()> {
        let choices = [
            (self.old, Selection::Old, "old", false),
            (self.old_orphan, Selection::OldOrphan, "old orphan", true),
            (self.orphan, Selection::Orphan, "orphan", true),
        ];
        for (chosen, selection, kind, is_orphan) in choices {
            if chosen && (!is_orphan || confirm_orphan_clean(directory_is_empty)?) {
                clean_crates(cache, kind, registry_dir, crate_list, selection)?;
            }
        }
        Ok(())
    }
}

/// clean crates of selection in cache and print total removed, kind names the
/// selection in the printed line
pub(super) fn clean_crates(
    cache: Cache,
    kind: &str,
    registry_dir: &mut RegistryDir,
    crate_list: &CrateList,
    selection: Selection,
) -> Result<()> {
    let (mut size, mut count, mut kept) = (0, 0, 0);
    if cache.has_registry() {
        let (registry_size, registry_count) =
            registry_dir.remove_crate_list(&crate_list.registry(selection))?;
        size += registry_size;
        count += registry_count;
        kept += crate_list.registry_kept(selection);
    }
    if cache.has_git() {
        let (git_size, git_count) = remove_git_crates(&crate_list.git(selection));
        size += git_size;
        count += git_count;
        kept += crate_list.git_kept(selection);
    }
    print_removed(&cache.crates_label(kind), (size, count), kept);
    Ok(())
}

/// remove sources of cache but keep archives so projects still build offline,
/// registry sources and index caches for registry and checkouts for git
pub(super) fn light_cleanup(cache: Cache, dir_path: &DirPath) {
    let mut success = true;
    if cache.has_registry() {
        success = delete_folder(dir_path.src_dir()).is_ok() && success;
        success = delete_index_cache(dir_path.index_dir()).is_ok() && success;
    }
    if cache.has_git() {
        success = delete_folder(dir_path.checkout_dir()).is_ok() && success;
    }
    if !success {
        println!("Failed to delete some folder during light cleanup");
    }
}
