use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::config_file::ConfigFile;

#[derive(Debug, StructOpt)]
#[structopt(about = "Initialize current working directory as cargo trim directory")]
pub(crate) struct Init {
    #[structopt(
        long = "dry-run",
        short = "n",
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
}

impl Init {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        config_file.add_directory(
            std::env::current_dir()
                .context("Current working directory is invalid")?
                .to_str()
                .context("failed to convert current directory to str")?,
            self.dry_run,
            true,
        )?;
        Ok(())
    }
}
