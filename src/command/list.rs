use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::{crate_list_type, crate_name_max_width};
use crate::list_crate::{CrateList, Selection};

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
        short = 'O',
        help = "List crates which are both old and orphan"
    )]
    old_orphan: bool,
    #[arg(long = "orphan", short = 'x', help = "List orphan crates")]
    orphan: bool,
    #[arg(
        long = "project",
        short = 'p',
        help = "List all detected Rust projects (directories containing a scanned Cargo.lock)"
    )]
    project: bool,
}

impl List {
    pub(super) fn run(
        &self,
        crate_list: &CrateList,
        source_url_max_width: usize,
        directory_is_empty: bool,
    ) {
        if self.all {
            crate_list_type(
                crate_list.bin(),
                source_url_max_width,
                crate_name_max_width(crate_list.bin()),
                "INSTALLED BIN",
            );
            list_selection(
                crate_list,
                Selection::All,
                "INSTALLED ",
                source_url_max_width,
            );
        }
        if self.old {
            list_selection(crate_list, Selection::Old, "OLD ", source_url_max_width);
        }
        if self.old_orphan {
            list_selection(
                crate_list,
                Selection::OldOrphan,
                "OLD+ORPHAN ",
                source_url_max_width,
            );
            // print warning if no directory present in config file
            if directory_is_empty {
                println!("{}", OLD_ORPHAN_LIST_WARNING.yellow());
            }
        }
        if self.orphan {
            list_selection(
                crate_list,
                Selection::Orphan,
                "ORPHAN ",
                source_url_max_width,
            );
            // print warning if directory config is empty
            if directory_is_empty {
                println!("{}", ORPHAN_LIST_WARNING.yellow());
            }
        }
        if self.project {
            list_projects(crate_list);
        }
    }
}

const OLD_ORPHAN_LIST_WARNING: &str =
    "WARNING: You have not initialized any directory as rust project directory. This will list \
     all old crates as old orphan crates even if they are not orphan crates. Run command 'cargo \
     trim set -d <directory>' to set rust project directory, use 'cargo trim set -d .' for \
     current directory";

const ORPHAN_LIST_WARNING: &str =
    "WARNING: You have not initialized any directory as rust project directory. This will list \
     all crates as orphan crate. Run command 'cargo trim set -d <directory>' to set rust project \
     directory, use 'cargo trim set -d .' for current directory";

/// list registry and git crates of a selection
fn list_selection(crate_list: &CrateList, selection: Selection, label: &str, first_width: usize) {
    let registry = crate_list.registry(selection);
    let git = crate_list.git(selection);
    let second_width = crate_name_max_width(registry.iter().chain(&git));
    crate_list_type(
        &registry,
        first_width,
        second_width,
        &format!("REGISTRY {label}CRATE"),
    );
    crate_list_type(
        &git,
        first_width,
        second_width,
        &format!("GIT {label}CRATE"),
    );
}

fn list_projects(crate_list: &CrateList) {
    let lock_files = crate_list.cargo_lock_files().paths();
    println!(
        "{}",
        format!("Total detected projects: {}", lock_files.len()).blue()
    );
    for (index, lock_file) in lock_files.iter().enumerate() {
        // A project is the directory that contains the detected Cargo.lock.
        let project_dir = lock_file.parent().unwrap_or(lock_file);
        println!(
            "{}: {}",
            format!("Project [{index}]").blue(),
            project_dir.display()
        );
    }
}
