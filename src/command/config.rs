use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;
use structopt::{clap::AppSettings, StructOpt};

use crate::config_file::ConfigFile;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Query about config file data used by CLI",
    settings=&[AppSettings::ArgRequiredElseHelp]
)]
pub(crate) struct Config {
    #[structopt(long = "directory", short = "d", help = "Query about directory data")]
    directory: bool,
    #[structopt(
        long = "ignore",
        short = "i",
        help = "Query about ignored file name data"
    )]
    ignore: bool,
    #[structopt(long = "location", short = "l", help = "Return config file location")]
    location: bool,
    #[structopt(long = "print", short = "p", help = "Display config file content")]
    print: bool,
}

impl Config {
    pub(super) fn run(&self, config_file: &ConfigFile, config_file_location: &Path) -> Result<()> {
        if self.directory {
            let read_directory = config_file.directory();
            for name in read_directory {
                println!("{}", name);
            }
        }
        if self.ignore {
            let read_ignore_file_name = config_file.ignore_file_name();
            for name in read_ignore_file_name {
                println!("{}", name);
            }
        }
        if self.location {
            println!(
                "{}: {:?}",
                "Config file location".color("blue"),
                config_file_location
            );
        }
        if self.print {
            let content = toml::to_string_pretty(config_file)
                .context("Failed to convert struct to pretty toml")?;
            println!("{}", content);
        }
        Ok(())
    }
}
