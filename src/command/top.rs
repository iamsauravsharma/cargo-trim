use anyhow::Result;
use clap::Parser;

use super::query::Parts;
use super::target::top_target;
use super::utils::{show_top_number_crates, source_name_max_width};
use crate::config_file::ConfigFile;
use crate::installed::Sources;
use crate::list_crate::{CrateList, Selection};

/// Show largest crates and target folders
///
/// Without any part option every part is shown, otherwise only the given parts
#[derive(Debug, Parser)]
pub(crate) struct Top {
    /// Number of largest items to show for each part
    #[arg(default_value_t = 10)]
    number: usize,
    #[command(flatten)]
    parts: Parts,
}

impl Top {
    pub(super) fn run(
        &self,
        crate_list: &CrateList,
        sources: &Sources,
        config_file: &ConfigFile,
    ) -> Result<()> {
        let max_width = source_name_max_width(sources);
        if self.parts.bin() {
            show_top_number_crates(crate_list.bin(), "bin", max_width, self.number);
        }
        if self.parts.registry() {
            let registry = crate_list.registry(Selection::Installed);
            show_top_number_crates(&registry, "registry", max_width, self.number);
        }
        if self.parts.git() {
            let git = crate_list.git(Selection::Installed);
            show_top_number_crates(&git, "git", max_width, self.number);
        }
        if self.parts.target() {
            top_target(config_file, self.number)?;
        }
        Ok(())
    }
}
