use std::path::Path;

use anyhow::{Context as _, Result};
use clap::Parser;

use crate::color::Paint as _;
use crate::config_file::ConfigFile;

/// Show config file location or content
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub(crate) struct Config {
    /// Print config file location
    #[arg(long, short)]
    location: bool,
    /// Print config file content
    #[arg(long, short)]
    print: bool,
}

impl Config {
    pub(super) fn run(&self, config_file: &ConfigFile, config_file_location: &Path) -> Result<()> {
        if self.location {
            println!(
                "{}: \"{}\"",
                "Config file location".blue(),
                config_file_location.display()
            );
        }
        if self.print {
            let content = toml::to_string_pretty(config_file)
                .context("failed to convert struct to pretty toml")?;
            println!("{content}");
        }
        Ok(())
    }
}
