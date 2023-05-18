use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::OwoColorize;

use crate::config_file::ConfigFile;

#[derive(Debug, Parser)]
#[command(
    about = "Query about config file data used by CLI",
    arg_required_else_help = true
)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Config {
    #[arg(long = "directory", short = 'd', help = "Query about directory data")]
    directory: bool,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Query about ignored file name data"
    )]
    ignore: bool,
    #[arg(long = "location", short = 'l', help = "Return config file location")]
    location: bool,
    #[arg(long = "print", short = 'p', help = "Display config file content")]
    print: bool,
}

impl Config {
    pub(super) fn run(&self, config_file: &ConfigFile, config_file_location: &Path) -> Result<()> {
        if self.directory {
            let read_directory = config_file.directory();
            for name in read_directory {
                println!("{name}");
            }
        }
        if self.ignore {
            let read_ignore_file_name = config_file.ignore_file_name();
            for name in read_ignore_file_name {
                println!("{name}");
            }
        }
        if self.location {
            println!(
                "{}: {config_file_location:?}",
                "Config file location".blue(),
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
