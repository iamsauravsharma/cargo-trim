use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[command(about = "Set config file values", arg_required_else_help = true)]
pub(crate) struct Set {
    #[arg(
        long = "directory",
        short = 'd',
        help = "Set directory of Rust project"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Add a relative or absolute path to ignore list in configuration file which is \
                ignored while scanning Cargo.lock file. A relative path is matched against the \
                trailing path components while an absolute path is matched against the full path",
        value_name = "path"
    )]
    ignore: Option<Vec<String>>,
    #[arg(
        long = "scan-hidden-folder",
        short = 'H',
        help = "Set whether hidden folder is scanned"
    )]
    scan_hidden_folder: Option<bool>,
    #[arg(
        long = "scan-target-folder",
        short = 'T',
        help = "Set whether target folder is scanned"
    )]
    scan_target_folder: Option<bool>,
    #[arg(
        long = "stale-days",
        short = 's',
        help = "Set number of days without any change after which a project is considered stale",
        value_name = "days"
    )]
    stale_days: Option<u32>,
}

impl Set {
    pub(super) fn run(&self, config_file: &mut ConfigFile, dry_run: bool) -> Result<()> {
        if let Some(directories) = &self.directory {
            for directory in directories {
                config_file.add_directory(directory, dry_run, true)?;
            }
        }
        if let Some(ignores) = &self.ignore {
            for ignore in ignores {
                // trim trailing separator so a path matches with or without it
                let path_separator = std::path::MAIN_SEPARATOR;
                let ignore = ignore.trim_end_matches(path_separator);
                config_file.add_ignore(ignore, dry_run, true)?;
            }
        }
        if let Some(scan_hidden_folder) = self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(scan_hidden_folder, dry_run, true)?;
        }
        if let Some(scan_target_folder) = self.scan_target_folder {
            config_file.set_scan_target_folder(scan_target_folder, dry_run, true)?;
        }
        if let Some(stale_days) = self.stale_days {
            config_file.set_stale_days(stale_days, dry_run, true)?;
        }

        Ok(())
    }
}
