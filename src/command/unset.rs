use anyhow::Result;
use clap::Parser;

use super::DryRun;
use crate::config_file::ConfigFile;
/// Remove config values
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub(crate) struct Unset {
    /// Remove project directory
    #[arg(long, short, value_name = "path")]
    directory: Option<Vec<String>>,
    #[command(flatten)]
    pub(super) dry_run: DryRun,
    /// Remove ignored path
    #[arg(long, short, value_name = "path")]
    ignore: Option<Vec<String>>,
    /// Remove crate filter
    #[arg(long, short, value_name = "spec")]
    filter: Option<Vec<String>>,
}

impl Unset {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        if let Some(directories) = &self.directory {
            for directory in directories {
                config_file.remove_directory(directory, true)?;
            }
        }
        if let Some(ignores) = &self.ignore {
            for ignore in ignores {
                let path_separator = std::path::MAIN_SEPARATOR;
                let ignore = ignore.trim_end_matches(path_separator);
                config_file.remove_ignore(ignore, true)?;
            }
        }
        if let Some(filters) = &self.filter {
            for filter in filters {
                config_file.remove_filter(filter, true)?;
            }
        }

        Ok(())
    }
}
