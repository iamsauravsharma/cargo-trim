use anyhow::Result;
use clap::Parser;

use super::clean::{Cache, CrateSelection, clean_crates, light_cleanup};
use super::utils::{count_in, print_dash};
use super::{DryRun, query_full_width, query_print};
use crate::dir_path::DirPath;
use crate::list_crate::{CrateList, Selection};
use crate::remove::RegistryDir;
use crate::utils::{convert_pretty, get_size};
/// Operate only on git cache
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub(crate) struct Git {
    /// Clean all git crates
    #[arg(long, short)]
    all: bool,
    #[command(flatten)]
    pub(super) dry_run: DryRun,
    /// Remove git checkouts but keep git db
    #[arg(long = "light", short)]
    light_cleanup: bool,
    #[command(flatten)]
    selection: CrateSelection,
}

impl Git {
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        registry_dir: &mut RegistryDir,
        directory_is_empty: bool,
    ) -> Result<()> {
        if self.light_cleanup {
            light_cleanup(Cache::Git, dir_path);
        }
        self.selection
            .clean(Cache::Git, registry_dir, crate_list, directory_is_empty)?;
        if self.all {
            clean_crates(Cache::Git, "", registry_dir, crate_list, Selection::All)?;
        }
        Ok(())
    }
}

pub(super) fn query_size_git(dir_path: &DirPath, crate_list: &CrateList) -> u64 {
    let installed = crate_list.git(Selection::Installed);
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
