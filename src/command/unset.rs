use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[command(about = "Remove config values", arg_required_else_help = true)]
pub(crate) struct Unset {
    #[arg(
        long = "directory",
        short = 'd',
        help = "Remove project directory",
        value_name = "path"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Remove ignored path",
        value_name = "path"
    )]
    ignore: Option<Vec<String>>,
    #[arg(
        long = "filter",
        short = 'f',
        help = "Remove crate filter",
        value_name = "spec"
    )]
    filter: Option<Vec<String>>,
}

impl Unset {
    pub(super) fn run(&self, config_file: &mut ConfigFile, dry_run: bool) -> Result<()> {
        if let Some(directories) = &self.directory {
            for directory in directories {
                config_file.remove_directory(directory, dry_run, true)?;
            }
        }
        if let Some(ignores) = &self.ignore {
            for ignore in ignores {
                let path_separator = std::path::MAIN_SEPARATOR;
                let ignore = ignore.trim_end_matches(path_separator);
                config_file.remove_ignore(ignore, dry_run, true)?;
            }
        }
        if let Some(filters) = &self.filter {
            for filter in filters {
                config_file.remove_filter(filter, dry_run, true)?;
            }
        }

        Ok(())
    }
}
