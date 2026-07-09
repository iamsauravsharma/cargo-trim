use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[command(about = "Set config file values", arg_required_else_help = true)]
pub(crate) struct Set {
    #[arg(
        long = "dry-run",
        short = 'n',
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
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
    #[arg(long = "scan-hidden-folder", help = "Set scan hidden folder as true")]
    scan_hidden_folder: bool,
    #[arg(long = "scan-target-folder", help = "Set scan hidden folder as true")]
    scan_target_folder: bool,
}

impl Set {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        let dry_run = self.dry_run;
        if let Some(directories) = &self.directory {
            for directory in directories {
                let path_separator = std::path::MAIN_SEPARATOR;
                let path = directory.trim_end_matches(path_separator);
                config_file.add_directory(path, dry_run, true)?;
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
        if self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(true, dry_run, true)?;
        }
        if self.scan_target_folder {
            config_file.set_scan_target_folder(true, dry_run, true)?;
        }

        Ok(())
    }
}
