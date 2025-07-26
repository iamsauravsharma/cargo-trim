use std::path::Path;

use anyhow::{Context as _, Result};
use clap::Parser;
use owo_colors::OwoColorize as _;

use crate::config_file::ConfigFile;

#[derive(Debug, Parser)]
#[command(
    about = "Query about config file data used by CLI",
    arg_required_else_help = true
)]
#[expect(clippy::struct_excessive_bools)]
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
            for (index, name) in read_directory.iter().enumerate() {
                println!("{}: {name}", format!("Directory [{index}]").blue());
            }
        }
        if self.ignore {
            let read_ignore_file_name = config_file.ignore_file_name();
            for (index, name) in read_ignore_file_name.iter().enumerate() {
                println!("{}: {name}", format!("Ignored name [{index}]").blue());
            }
        }
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
