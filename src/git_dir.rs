use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::utils::delete_folder;

/// Store git dir folder information
pub(crate) struct GitDir;

impl GitDir {
    /// remove crates
    fn remove_crate(
        crate_detail: &CrateDetail,
        crate_metadata: &CrateMetaData,
        dry_run: bool,
    ) -> Result<bool> {
        let is_success = if crate_metadata.name().contains("-HEAD") {
            if let Some(found_crate_metadata) = crate_detail
                .git_crates_archive()
                .iter()
                .find(|&source_metadata| source_metadata == crate_metadata)
            {
                let path = found_crate_metadata
                    .path()
                    .clone()
                    .context("expected path from crate detail metadata")?;
                delete_folder(&path, dry_run).is_ok()
            } else {
                true
            }
        } else if let Some(found_crate_metadata) = crate_detail
            .git_crates_source()
            .iter()
            .find(|&source_metadata| source_metadata == crate_metadata)
        {
            let path = found_crate_metadata
                .path()
                .clone()
                .context("expected path from crate detail metadata")?;
            delete_folder(&path, dry_run).is_ok()
        } else {
            true
        };
        if dry_run {
            println!(
                "{} {} {} {}",
                "Dry run:".yellow(),
                "Removed".red(),
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata.name(),
            );
            Ok(true)
        } else if is_success {
            println!(
                "{} {} {}",
                "Removed".red(),
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata.name()
            );
            Ok(true)
        } else {
            println!(
                r#"Failed to remove {} {}"#,
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata.name(),
            );
            Ok(false)
        }
    }

    /// Remove list of crates
    pub(crate) fn remove_crate_list(
        crate_detail: &CrateDetail,
        list: &[CrateMetaData],
        dry_run: bool,
    ) -> Result<(u64, usize)> {
        let mut size_cleaned = 0;
        let mut crate_removed = 0;
        for crate_metadata in list {
            if Self::remove_crate(crate_detail, crate_metadata, dry_run)? {
                size_cleaned += crate_metadata.size();
                crate_removed += 1;
            }
        }
        Ok((size_cleaned, crate_removed))
    }
}
