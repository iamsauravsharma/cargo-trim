use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::crate_list_type;
use crate::list_crate::CrateList;

#[derive(Debug, Parser)]
#[command(about = "List crates", arg_required_else_help = true)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct List {
    #[arg(long = "all", short = 'a', help = "List all installed crate")]
    all: bool,
    #[arg(long = "old", short = 'o', help = "List old crates")]
    old: bool,
    #[arg(
        long = "old-orphan",
        short = 'z',
        help = "List crates which are both old and orphan"
    )]
    old_orphan: bool,
    #[arg(long = "orphan", short = 'x', help = "List orphan crates")]
    orphan: bool,
}

impl List {
    pub(super) fn run(
        &self,
        crate_list: &CrateList,
        source_url_max_width: usize,
        directory_is_empty: bool,
    ) {
        if self.all {
            list_all(crate_list, source_url_max_width);
        }
        if self.old {
            list_old(crate_list, source_url_max_width);
        }
        if self.old_orphan {
            list_old_orphan(crate_list, source_url_max_width, directory_is_empty);
        }
        if self.orphan {
            list_orphan(crate_list, source_url_max_width, directory_is_empty);
        }
    }
}

fn list_all(crate_list: &CrateList, first_width: usize) {
    let second_width = std::cmp::max(
        crate_list
            .installed_bin()
            .iter()
            .chain(crate_list.installed_registry())
            .chain(crate_list.installed_git())
            .map(|cm| {
                if let Some(version) = cm.version() {
                    cm.name().len() + version.to_string().len() + 1
                } else {
                    cm.name().len()
                }
            })
            .max()
            .unwrap_or(30),
        30,
    ) + 2;
    crate_list_type(
        crate_list.installed_bin(),
        first_width,
        second_width,
        "INSTALLED BIN",
    );
    crate_list_type(
        crate_list.installed_registry(),
        first_width,
        second_width,
        "REGISTRY INSTALLED CRATE",
    );
    crate_list_type(
        crate_list.installed_git(),
        first_width,
        second_width,
        "GIT INSTALLED CRATE",
    );
}

fn list_old(crate_list: &CrateList, first_width: usize) {
    let second_width = std::cmp::max(
        crate_list
            .old_registry()
            .iter()
            .chain(crate_list.old_git())
            .map(|cm| {
                if let Some(version) = cm.version() {
                    cm.name().len() + version.to_string().len() + 1
                } else {
                    cm.name().len()
                }
            })
            .max()
            .unwrap_or(30),
        30,
    ) + 2;
    crate_list_type(
        crate_list.old_registry(),
        first_width,
        second_width,
        "REGISTRY OLD CRATE",
    );
    crate_list_type(
        crate_list.old_git(),
        first_width,
        second_width,
        "GIT OLD CRATE",
    );
}

fn list_old_orphan(crate_list: &CrateList, first_width: usize, directory_is_empty: bool) {
    let second_width = std::cmp::max(
        crate_list
            .old_orphan_registry()
            .iter()
            .chain(&crate_list.old_orphan_git())
            .map(|cm| {
                if let Some(version) = cm.version() {
                    cm.name().len() + version.to_string().len() + 1
                } else {
                    cm.name().len()
                }
            })
            .max()
            .unwrap_or(30),
        30,
    ) + 2;
    crate_list_type(
        &crate_list.old_orphan_registry(),
        first_width,
        second_width,
        "REGISTRY OLD+ORPHAN CRATE",
    );
    crate_list_type(
        &crate_list.old_orphan_git(),
        first_width,
        second_width,
        "GIT OLD+ORPHAN CRATE",
    );
    // print warning if no directory present in config file
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list all old crates as old orphan crates even if \
                            they are not orphan crates. Run command 'cargo trim init' to \
                            initialize current directory as rust project directory or pass cargo \
                            trim set -d <directory> for setting rust project directory";
        println!("{}", warning_text.yellow());
    }
}

fn list_orphan(crate_list: &CrateList, first_width: usize, directory_is_empty: bool) {
    let second_width = std::cmp::max(
        crate_list
            .orphan_registry()
            .iter()
            .chain(crate_list.orphan_git())
            .map(|cm| {
                if let Some(version) = cm.version() {
                    cm.name().len() + version.to_string().len() + 1
                } else {
                    cm.name().len()
                }
            })
            .max()
            .unwrap_or(30),
        30,
    ) + 2;
    crate_list_type(
        crate_list.orphan_registry(),
        first_width,
        second_width,
        "REGISTRY ORPHAN CRATE",
    );
    crate_list_type(
        crate_list.orphan_git(),
        first_width,
        second_width,
        "GIT ORPHAN CRATE",
    );
    // print warning if directory config is empty
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list all crates as orphan crate. Run command \
                            'cargo trim init' to initialize current directory as rust project \
                            directory or pass cargo trim set -d <directory> for setting rust \
                            project directory";
        println!("{}", warning_text.yellow());
    }
}
