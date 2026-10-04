use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[command(about = "Unset values from config file", arg_required_else_help = true)]
pub(crate) struct Unset {
    #[arg(
        long = "directory",
        short = 'd',
        help = "Directory to be removed from config file"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Relative or absolute path to be removed from ignore list in config file",
        value_name = "path"
    )]
    ignore: Option<Vec<String>>,
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

        Ok(())
    }
}
