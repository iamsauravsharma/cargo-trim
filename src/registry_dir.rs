use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::utils::delete_folder;

/// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir<'a> {
    cache_dir: &'a str,
    src_dir: &'a str,
    index_cache_dir: Vec<String>,
    installed_crate: Vec<CrateMetaData>,
}

impl<'a> RegistryDir<'a> {
    /// Create new registry dir
    pub(crate) fn new(
        cache_dir: &'a Path,
        src_dir: &'a Path,
        index_dir: &Path,
        installed_crate: &[CrateMetaData],
    ) -> Result<Self> {
        let cache_dir = cache_dir.to_str().unwrap();
        let src_dir = src_dir.to_str().unwrap();
        let mut index_cache_dir = Vec::new();
        // read a index .cache dir folder for each registry by analyzing index folder
        if index_dir.exists() {
            for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
                let entry = entry?.path();
                for folder in fs::read_dir(entry).context("failed to read registry directory")? {
                    let folder = folder?.path();
                    let folder_name = folder
                        .file_name()
                        .context("failed to get file name form registry sub directory")?;
                    if folder_name == ".cache" {
                        index_cache_dir.push(folder.to_str().unwrap().to_string());
                    }
                }
            }
        }

        Ok(Self {
            cache_dir,
            src_dir,
            index_cache_dir,
            installed_crate: installed_crate.to_owned(),
        })
    }

    /// Remove crate from src & cache directory
    pub(crate) fn remove_crate(
        &mut self,
        crate_detail: &CrateDetail,
        crate_metadata: &CrateMetaData,
        dry_run: bool,
    ) -> bool {
        // remove crate from cache dir
        let mut is_success = remove_crate(
            Path::new(&self.cache_dir),
            crate_detail,
            crate_metadata,
            dry_run,
        )
        .is_ok();

        // remove crate from index dir
        is_success = remove_crate(
            Path::new(&self.src_dir),
            crate_detail,
            crate_metadata,
            dry_run,
        )
        .is_ok()
            && is_success;

        let index_cache = self.index_cache_dir.clone();

        // remove index cache dir if their is only one crate. It will also clean crate
        // name from installed crate name owned locally by it so when two version of
        // same crate is deleted it properly remove index cache
        for index_cache_dir in &index_cache {
            let index = Path::new(&index_cache_dir);
            let source = crate_detail
                .source_url_from_path(index.parent().unwrap())
                .unwrap();
            if &Some(source) == crate_metadata.source() {
                let same_name_list = self.installed_crate.iter().filter(|&x| {
                    x.name() == crate_metadata.name() && x.source() == crate_metadata.source()
                });
                if same_name_list.count() == 1 {
                    is_success =
                        remove_index_cache(index, crate_metadata, dry_run).is_ok() && is_success;
                }
                is_success = remove_empty_index_cache_dir(index, dry_run).is_ok() && is_success;
                self.installed_crate.retain(|x| x != crate_metadata);
            }
        }
        if dry_run {
            println!(
                r#"{} {} "{}-{}""#,
                "Dry run:".yellow(),
                "Removed".red(),
                crate_metadata.name(),
                crate_metadata.version().as_ref().unwrap(),
            );
            true
        } else if is_success {
            println!(
                r#"{} "{}-{}""#,
                "Removed".red(),
                crate_metadata.name(),
                crate_metadata.version().as_ref().unwrap(),
            );
            true
        } else {
            println!(
                r#"Failed to remove "{}-{}""#,
                crate_metadata.name(),
                crate_metadata.version().as_ref().unwrap(),
            );
            false
        }
    }

    /// Remove list of crates
    pub(crate) fn remove_crate_list(
        &mut self,
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

/// Remove crates which name is provided to delete
fn remove_crate(
    path: &Path,
    crate_detail: &CrateDetail,
    crate_metadata: &CrateMetaData,
    dry_run: bool,
) -> Result<()> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            let source = crate_detail.source_url_from_path(&path)?;
            if &Some(source) == crate_metadata.source() {
                for entry in fs::read_dir(path)? {
                    let path = entry?.path();
                    if path.to_str().unwrap().contains(crate_metadata.name()) {
                        delete_folder(&path, dry_run)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// determine crate index cache location and remove crate index cache
fn remove_index_cache(path: &Path, crate_metadata: &CrateMetaData, dry_run: bool) -> Result<()> {
    let mut crate_index_cache_location = path.to_path_buf();
    let name = crate_metadata.name();
    match name.len() {
        1 => {
            crate_index_cache_location.push("1");
            crate_index_cache_location.push(name);
        }
        2 => {
            crate_index_cache_location.push("2");
            crate_index_cache_location.push(name);
        }
        3 => {
            crate_index_cache_location.push("3");
            crate_index_cache_location.push(&name[..1]);
            crate_index_cache_location.push(name);
        }
        _ => {
            crate_index_cache_location.push(&name[..2]);
            crate_index_cache_location.push(&name[2..4]);
            crate_index_cache_location.push(name);
        }
    };
    delete_folder(&crate_index_cache_location, dry_run)?;
    Ok(())
}

/// check if any index cache folder is empty if it is it is removed out
fn remove_empty_index_cache_dir(path: &Path, dry_run: bool) -> Result<()> {
    if fs::read_dir(path)
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
    {
        delete_folder(path, dry_run)?;
    } else {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                remove_empty_index_cache_dir(&path, dry_run)?;
            }
        }
    }
    Ok(())
}
