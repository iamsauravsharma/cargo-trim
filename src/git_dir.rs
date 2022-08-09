use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::utils::delete_folder;

/// Store git dir folder information
pub(crate) struct GitDir<'a> {
    checkout_dir: &'a str,
    db_dir: &'a str,
}

impl<'a> GitDir<'a> {
    /// create new git dir
    pub(crate) fn new(checkout_dir: &'a Path, db_dir: &'a Path) -> Result<Self> {
        let checkout_dir = checkout_dir
            .to_str()
            .context("Failed checkout dir path conversion")?;
        let db_dir = db_dir.to_str().context("Failed db dir path conversion")?;
        Ok(Self {
            checkout_dir,
            db_dir,
        })
    }

    /// remove crates
    pub(crate) fn remove_crate(
        &self,
        crate_detail: &CrateDetail,
        crate_metadata: &CrateMetaData,
        dry_run: bool,
    ) -> bool {
        let is_success = if crate_metadata.name().contains("-HEAD") {
            remove_crate(
                Path::new(&self.db_dir),
                crate_detail,
                crate_metadata,
                dry_run,
            )
            .is_ok()
        } else {
            remove_crate(
                Path::new(&self.checkout_dir),
                crate_detail,
                crate_metadata,
                dry_run,
            )
            .is_ok()
        };
        if dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".yellow(),
                "Removed".red(),
                crate_metadata.name(),
            );
            true
        } else if is_success {
            println!("{} {:?}", "Removed".red(), crate_metadata.name(),);
            true
        } else {
            println!(r#"Failed to remove {:?}"#, crate_metadata.name(),);
            false
        }
    }

    /// Remove list of crates
    pub(crate) fn remove_crate_list(
        &self,
        crate_detail: &CrateDetail,
        list: &[CrateMetaData],
        dry_run: bool,
    ) -> (u64, usize) {
        let mut size_cleaned = 0;
        let mut crate_removed = 0;
        for crate_metadata in list {
            if self.remove_crate(crate_detail, crate_metadata, dry_run) {
                size_cleaned += crate_metadata.size();
                crate_removed += 1;
            }
        }
        (size_cleaned, crate_removed)
    }
}

/// preform remove operation
fn remove_crate(
    location: &Path,
    crate_detail: &CrateDetail,
    crate_metadata: &CrateMetaData,
    dry_run: bool,
) -> Result<()> {
    for entry in fs::read_dir(location)? {
        let path = entry?.path();
        let source = crate_detail.source_url_from_path(&path)?;
        if &Some(source) == crate_metadata.source() {
            // split name to split crate and rev sha
            let name = crate_metadata.name();
            let name = name.rsplitn(2, '-').collect::<Vec<&str>>();
            let crate_name = name[1];
            let rev_sha = name[0];
            if path
                .to_str()
                .context("failed git directory crate path to str")?
                .contains(crate_name)
            {
                if rev_sha.contains("HEAD") {
                    delete_folder(&path, dry_run)?;
                } else {
                    for rev in fs::read_dir(&path)? {
                        let path = rev?.path();
                        let file_name = path
                            .file_name()
                            .context("Failed to get file name to check rev sha")?
                            .to_str()
                            .context("Failed rev sha file name to str conversion")?;
                        if file_name == rev_sha {
                            delete_folder(&path, dry_run)?;
                        }
                    }
                    if fs::read_dir(&path)?.next().is_none() {
                        delete_folder(&path, dry_run)?;
                    }
                }
            }
        }
    }
    Ok(())
}
