use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::config_file::ConfigFile;

#[derive(Debug, StructOpt)]
#[structopt(about = "Clear current working directory from cargo cache config")]
pub(crate) struct Clear {
    #[structopt(
        long = "dry-run",
        short = "n",
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
}

impl Clear {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        config_file.remove_directory(
            std::env::current_dir()
                .context("Current working directory is invalid")?
                .to_str()
                .context("Cannot convert current directory to str")?,
            self.dry_run,
        )?;
        Ok(())
    }
}
