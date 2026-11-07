use std::collections::HashSet;

use anyhow::Result;
use clap::{Args, Parser};

use super::git::query_size_git;
use super::registry::query_size_registry;
use super::target::query_size_target;
use super::utils::{print_dash, query_full_width, query_print};
use crate::config_file::ConfigFile;
use crate::dir_path::DirPath;
use crate::list_crate::CrateList;
use crate::utils::{convert_pretty, get_inode_handled_size};

/// Show size of cache and target folders
///
/// Without any part option every part is shown, otherwise only the given parts
#[derive(Debug, Parser)]
pub(crate) struct Query {
    #[command(flatten)]
    parts: Parts,
}

/// Parts of cache and projects a command shows, shared by `query` and `top`
#[derive(Debug, Args)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent command line flag"
)]
pub(super) struct Parts {
    /// Include .cargo/bin binaries
    #[arg(long, short)]
    bin: bool,
    /// Include git cache
    #[arg(long, short)]
    git: bool,
    /// Include registry cache
    #[arg(long, short)]
    registry: bool,
    /// Include target folders of projects
    #[arg(long, short)]
    target: bool,
}

impl Parts {
    /// no part given means every part
    fn every(&self) -> bool {
        !(self.bin || self.git || self.registry || self.target)
    }

    pub(super) fn bin(&self) -> bool {
        self.bin || self.every()
    }

    pub(super) fn git(&self) -> bool {
        self.git || self.every()
    }

    pub(super) fn registry(&self) -> bool {
        self.registry || self.every()
    }

    pub(super) fn target(&self) -> bool {
        self.target || self.every()
    }
}

impl Query {
    pub(super) fn run(
        &self,
        dir_path: &DirPath,
        crate_list: &CrateList,
        config_file: &ConfigFile,
    ) -> Result<()> {
        let mut parts = Vec::new();
        if self.parts.bin() {
            parts.push(query_size_bin(dir_path, crate_list));
        }
        if self.parts.registry() {
            parts.push(query_size_registry(dir_path, crate_list));
        }
        if self.parts.git() {
            parts.push(query_size_git(dir_path, crate_list));
        }
        // a single part already shows its own total
        if parts.len() > 1 {
            query_print("Total size", &convert_pretty(parts.iter().sum()));
            print_dash(query_full_width());
        }
        if self.parts.target() {
            query_size_target(config_file)?;
            print_dash(query_full_width());
        }
        Ok(())
    }
}

/// print and return size of .cargo/bin binaries
fn query_size_bin(dir_path: &DirPath, crate_list: &CrateList) -> u64 {
    let size = get_inode_handled_size(dir_path.bin_dir(), &mut HashSet::new()).unwrap_or(0_u64);
    query_print(
        &format!(
            "Total size of {} .cargo/bin binary:",
            crate_list.bin().len()
        ),
        &convert_pretty(size),
    );
    print_dash(query_full_width());
    size
}
