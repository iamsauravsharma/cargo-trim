use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[clap(about = "Unset values from config file", arg_required_else_help = true)]
pub(crate) struct Unset {
    #[clap(
        long = "dry-run",
        short = 'n',
        help = "Run command in dry run mode to see what would be done"
    )]
    dry_run: bool,
    #[clap(
        long = "directory",
        short = 'd',
        help = "Directory to be removed from config file"
    )]
    directory: Option<Vec<String>>,
    #[clap(
        long = "ignore",
        short = 'i',
        help = "Ignore file name to be removed from config file",
        value_name = "file"
    )]
    ignore: Option<Vec<String>>,
    #[clap(long = "scan-hidden-folder", help = "Set scan hidden folder as false")]
    scan_hidden_folder: bool,
    #[clap(long = "scan-target-folder", help = "Set scan hidden folder as false")]
    scan_target_folder: bool,
}

impl Unset {
    pub(super) fn run(&self, config_file: &mut ConfigFile) -> Result<()> {
        let dry_run = self.dry_run;
        if let Some(directories) = &self.directory {
            for directory in directories {
                let path_separator = std::path::MAIN_SEPARATOR;
                let path = directory.trim_end_matches(path_separator);
                config_file.remove_directory(path, dry_run, true)?;
            }
        }
        if let Some(files) = &self.ignore {
            for file in files {
                config_file.remove_ignore_file_name(file, dry_run, true)?;
            }
        }
        if self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(false, dry_run, true)?;
        }
        if self.scan_target_folder {
            config_file.set_scan_target_folder(false, dry_run, true)?;
        }

        Ok(())
    }
}
