use clap::Parser;
use owo_colors::OwoColorize as _;

use super::utils::{crate_list_type, crate_name_max_width};
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
        if self.project {
            list_projects(crate_list);
        }
    }
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

fn list_all(crate_list: &CrateList, first_width: usize) {
    let second_width = crate_name_max_width(
        crate_list
            .installed_bin()
            .iter()
            .chain(crate_list.installed_registry())
            .chain(crate_list.installed_git()),
    );
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
    let second_width =
        crate_name_max_width(crate_list.old_registry().iter().chain(crate_list.old_git()));
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
    let old_orphan_registry = crate_list.old_orphan_registry();
    let old_orphan_git = crate_list.old_orphan_git();
    let second_width = crate_name_max_width(old_orphan_registry.iter().chain(&old_orphan_git));
    crate_list_type(
        &old_orphan_registry,
        first_width,
        second_width,
        "REGISTRY OLD+ORPHAN CRATE",
    );
    crate_list_type(
        &old_orphan_git,
        first_width,
        second_width,
        "GIT OLD+ORPHAN CRATE",
    );
    // print warning if no directory present in config file
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list all old crates as old orphan crates even if \
                            they are not orphan crates. Run command 'cargo trim set -d \
                            <directory>' to set rust project directory, use 'cargo trim set -d .' \
                            for current directory";
        println!("{}", warning_text.yellow());
    }
}

fn list_orphan(crate_list: &CrateList, first_width: usize, directory_is_empty: bool) {
    let second_width = crate_name_max_width(
        crate_list
            .orphan_registry()
            .iter()
            .chain(crate_list.orphan_git()),
    );
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
                            'cargo trim set -d <directory>' to set rust project directory, use \
                            'cargo trim set -d .' for current directory";
        println!("{}", warning_text.yellow());
    }
}
