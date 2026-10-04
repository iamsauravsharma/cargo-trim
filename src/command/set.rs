use anyhow::Result;
use clap::Parser;

use crate::config_file::ConfigFile;
#[derive(Debug, Parser)]
#[command(about = "Add or change config values", arg_required_else_help = true)]
pub(crate) struct Set {
    #[arg(
        long = "directory",
        short = 'd',
        help = "Add project directory",
        long_help = "Add project directory. Add directory scanned for rust projects, a relative \
                     path such as `.` is stored as absolute path",
        value_name = "path"
    )]
    directory: Option<Vec<String>>,
    #[arg(
        long = "ignore",
        short = 'i',
        help = "Add path ignored while scanning",
        long_help = "Add path ignored while scanning. A relative path matches trailing path \
                     components anywhere, an absolute path matches the full path and each \
                     component may contain `*`. A folder holding `.cargo-trim-ignore` is always \
                     skipped",
        value_name = "path"
    )]
    ignore: Option<Vec<String>>,
    #[arg(
        long = "filter",
        short = 'f',
        help = "Add crate filter",
        long_help = "Add crate filter. Package spec `[registry:]name[@version]`. Registry and \
                     name may contain `*`, registry matches the source folder name with or \
                     without its hash suffix and version is a semver requirement (exact for a \
                     full version) or a git revision prefix. Without any plain entry every crate \
                     may be cleaned, otherwise only matching crates are cleaned. An entry \
                     starting with `!` is never cleaned and wins over other entries",
        value_name = "spec"
    )]
    filter: Option<Vec<String>>,
    #[arg(
        long = "scan-hidden-folder",
        short = 'H',
        help = "Set whether hidden folders are scanned",
        value_name = "bool"
    )]
    scan_hidden_folder: Option<bool>,
    #[arg(
        long = "scan-target-folder",
        short = 'T',
        help = "Set whether target folders are scanned for Cargo.lock files",
        value_name = "bool"
    )]
    scan_target_folder: Option<bool>,
    #[arg(
        long = "stale-days",
        short = 's',
        help = "Set days without change after which a project is stale",
        long_help = "Set days without change after which a project is stale. Projects without any \
                     change for this many days are stale and their Cargo.lock files no longer \
                     mark crates as used, 0 turns it off",
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
        if let Some(filters) = &self.filter {
            for filter in filters {
                config_file.add_filter(filter, dry_run, true)?;
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
