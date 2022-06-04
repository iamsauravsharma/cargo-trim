use clap::Parser;
use owo_colors::OwoColorize;

use crate::crate_detail::CrateDetail;
use crate::list_crate::CrateList;

#[derive(Debug, Parser)]
#[clap(about = "List out crates", arg_required_else_help = true)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct List {
    #[clap(long = "all", short = 'a', help = "List out all installed crate")]
    all: bool,
    #[clap(long = "old", short = 'o', help = "List out old crates")]
    old: bool,
    #[clap(
        long = "old-orphan",
        short = 'z',
        help = "List out crates which are both old and orphan"
    )]
    old_orphan: bool,
    #[clap(long = "orphan", short = 'x', help = "List out orphan crates")]
    orphan: bool,
    #[clap(long = "used", short = 'u', help = "List out used crates")]
    used: bool,
}

impl List {
    pub(super) fn run(
        &self,
        crate_detail: &CrateDetail,
        crate_list: &CrateList,
        directory_is_empty: bool,
    ) {
        if self.all {
            list_all(crate_detail, crate_list);
        }
        if self.old {
            list_old(crate_detail, crate_list);
        }
        if self.old_orphan {
            list_old_orphan(crate_detail, crate_list, directory_is_empty);
        }
        if self.orphan {
            list_orphan(crate_detail, crate_list, directory_is_empty);
        }
        if self.used {
            list_used(crate_detail, crate_list, directory_is_empty);
        }
    }
}

fn list_all(crate_detail: &CrateDetail, crate_list: &CrateList) {
    crate_list_type(
        crate_detail,
        crate_list.installed_registry(),
        "REGISTRY INSTALLED CRATE",
    );
    crate_list_type(
        crate_detail,
        crate_list.installed_git(),
        "GIT INSTALLED CRATE",
    );
}

fn list_old(crate_detail: &CrateDetail, crate_list: &CrateList) {
    crate_list_type(
        crate_detail,
        crate_list.old_registry(),
        "REGISTRY OLD CRATE",
    );
    crate_list_type(crate_detail, crate_list.old_git(), "GIT OLD CRATE");
}

fn list_old_orphan(crate_detail: &CrateDetail, crate_list: &CrateList, directory_is_empty: bool) {
    crate_list_type(
        crate_detail,
        &crate_list.list_old_orphan_registry(),
        "REGISTRY OLD+ORPHAN CRATE",
    );
    crate_list_type(
        crate_detail,
        &crate_list.list_old_orphan_git(),
        "GIT OLD+ORPHAN CRATE",
    );
    // print waning if no directory present in config file
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list all old crates as old orphan crates even if \
                            they are not orphan crates. Run command 'cargo trim init' to \
                            initialize current directory as rust project directory or pass cargo \
                            trim -d <directory> for setting rust project directory";
        println!("{}", warning_text.yellow());
    }
}

fn list_orphan(crate_detail: &CrateDetail, crate_list: &CrateList, directory_is_empty: bool) {
    crate_list_type(
        crate_detail,
        crate_list.orphan_registry(),
        "REGISTRY ORPHAN CRATE",
    );
    crate_list_type(crate_detail, crate_list.orphan_git(), "GIT ORPHAN CRATE");
    // print warning if directory config is empty
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list all crates as orphan crate. Run command \
                            'cargo trim init' to initialize current directory as rust project \
                            directory or pass cargo trim -d <directory> for setting rust project \
                            directory";
        println!("{}", warning_text.yellow());
    }
}

fn list_used(crate_detail: &CrateDetail, crate_list: &CrateList, directory_is_empty: bool) {
    crate_list_type(
        crate_detail,
        crate_list.used_registry(),
        "REGISTRY USED CRATE",
    );
    crate_list_type(crate_detail, crate_list.used_git(), "GIT USED CRATE");
    // print warning if directory config is empty
    if directory_is_empty {
        let warning_text = "WARNING: You have not initialized any directory as rust project \
                            directory. This will list no crates as used crate. Run command 'cargo \
                            trim init' to initialize current directory as rust project directory \
                            or pass cargo trim -d <directory> for setting rust project directory";
        println!("{}", warning_text.yellow());
    }
}

// list certain crate type to terminal
fn crate_list_type(crate_detail: &CrateDetail, crate_type: &[String], title: &str) {
    let first_width = 40;
    let second_width = 10;
    let precision = 3;
    let dash_len = first_width + second_width + 3;
    crate::utils::show_title(title, first_width, second_width, dash_len);

    let mut total_size = 0.0;
    for crates in crate_type {
        let size = crate_detail.find(crates, title);
        total_size += size;
        println!(
            "|{:^first_width$}|{:^second_width$.precision$}|",
            crates, size
        );
    }
    crate::utils::show_total_count(crate_type, total_size, first_width, second_width, dash_len);
}
