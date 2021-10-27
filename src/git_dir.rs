use std::{fs, path::Path};

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::{crate_detail::CrateDetail, utils::delete_folder};

// Store git dir folder information
pub(crate) struct GitDir<'a> {
    checkout_dir: &'a str,
    db_dir: &'a str,
}

impl<'a> GitDir<'a> {
    // create new GitDir
    pub(crate) fn new(checkout_dir: &'a Path, db_dir: &'a Path) -> Self {
        let checkout_dir = checkout_dir.to_str().unwrap();
        let db_dir = db_dir.to_str().unwrap();
        Self {
            checkout_dir,
            db_dir,
        }
    }

    // remove crates
    pub(crate) fn remove_crate(&self, crate_name: &str, dry_run: bool) {
        let is_success;
        if crate_name.contains("-HEAD") {
            is_success = remove_crate(Path::new(&self.db_dir), crate_name, dry_run).is_ok();
        } else {
            is_success = remove_crate(Path::new(&self.checkout_dir), crate_name, dry_run).is_ok();
        }
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".yellow(),
                "Removed".red(),
                crate_name
            );
        } else if is_success {
            println!("{} {:?}", "Removed".red(), crate_name);
        } else {
            println!("Failed to remove {:?}", crate_name);
        }
    }

    // Remove list of crates
    pub(crate) fn remove_crate_list(
        &self,
        crate_detail: &CrateDetail,
        list: &[String],
        dry_run: bool,
    ) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name, dry_run);
            size_cleaned += crate_detail.find(crate_name, "GIT");
        }
        size_cleaned
    }
}

// preform remove operation
fn remove_crate(location: &Path, crate_name: &str, dry_run: bool) -> Result<()> {
    for entry in fs::read_dir(location)? {
        let path = entry?.path();
        // split directory name to split crate and rev sha
        let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
        let crate_name = name[1];
        let rev_sha = name[0];
        if path.to_str().unwrap().contains(crate_name) {
            if rev_sha.contains("HEAD") {
                delete_folder(&path, dry_run)?;
            } else {
                for rev in fs::read_dir(path)? {
                    let path = rev?.path();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    if file_name == rev_sha {
                        delete_folder(&path, dry_run)?;
                    }
                }
            }
        }
    }
    Ok(())
}
