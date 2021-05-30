use anyhow::Result;
use structopt::{clap::AppSettings, StructOpt};

use crate::config_file::ConfigFile;
#[derive(Debug, StructOpt)]
#[structopt(about = "Remove values from config file", settings=&[AppSettings::ArgRequiredElseHelp])]
pub(crate) struct Remove {
    #[structopt(
        long = "dry-run",
        short = "n",
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[structopt(
        long = "directory",
        short = "d",
        help = "Directory to be removed from config file"
    )]
    directory: Option<Vec<String>>,
    #[structopt(
        long = "ignore",
        short = "i",
        help = "Ignore file name to be removed from config file",
        value_name = "file"
    )]
    ignore: Option<Vec<String>>,
}

impl Remove {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        let dry_run = self.dry_run;
        if let Some(directories) = &self.directory {
            for directory in directories {
                let path_separator = std::path::MAIN_SEPARATOR;
                let path = directory.trim_end_matches(path_separator);
                config_file.remove_directory(path, dry_run)?;
            }
        }
        if let Some(files) = &self.ignore {
            for file in files {
                config_file.remove_ignore_file_name(file, dry_run)?;
            }
        }

        Ok(())
    }
}
