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
        let cache_dir_str = cache_dir
            .to_str()
            .context("failed to convert cache dir to str")?;
        let src_dir_str = src_dir
            .to_str()
            .context("failed to convert src dir to str")?;
        let mut index_cache_dir = Vec::new();
        // read a index .cache dir folder for each registry by analyzing index folder
        if index_dir.exists() && index_dir.is_dir() {
            for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
                let mut entry_path = entry?.path();
                entry_path.push(".cache");
                if entry_path.exists() {
                    index_cache_dir.push(
                        entry_path
                            .to_str()
                            .context("unable to convert index cache folder to str")?
                            .to_string(),
                    );
                }
            }
        }

        Ok(Self {
            cache_dir: cache_dir_str,
            src_dir: src_dir_str,
            index_cache_dir,
            installed_crate: installed_crate.to_owned(),
        })
    }

    /// Remove crate from src & cache directory
    fn remove_crate(
        &mut self,
        crate_detail: &CrateDetail,
        crate_metadata: &CrateMetaData,
        dry_run: bool,
    ) -> Result<bool> {
        // remove crate from cache dir
        let mut is_success = remove_crate_from_location(
            Path::new(&self.cache_dir),
            crate_detail,
            crate_metadata,
            dry_run,
        )
        .is_ok();

        // remove crate from index dir
        is_success = remove_crate_from_location(
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
                .source_url_from_path(index.parent().context("failed to get index parent")?)?;
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
                "{} {} {} {}-{}",
                "Dry run:".yellow(),
                "Removed".red(),
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata.name(),
                crate_metadata
                    .version()
                    .clone()
                    .context("failed to convert crate version")?,
            );
            Ok(true)
        } else if is_success {
            println!(
                "{} {} {}-{}",
                "Removed".red(),
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata.name(),
                crate_metadata
                    .version()
                    .clone()
                    .context("failed to convert crate version")?,
            );
            Ok(true)
        } else {
            println!(
                "Failed to remove {} {}-{}",
                crate_metadata.name(),
                crate_metadata
                    .source()
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                crate_metadata
                    .version()
                    .clone()
                    .context("failed to convert crate version")?,
            );
            Ok(false)
        }
    }

    /// Remove list of crates
    pub(crate) fn remove_crate_list(
        &mut self,
        crate_detail: &CrateDetail,
        crate_metadata_list: &[CrateMetaData],
        dry_run: bool,
    ) -> Result<(u64, usize)> {
        let mut size_cleaned = 0;
        let mut crate_removed = 0;
        for crate_metadata in crate_metadata_list {
            if self.remove_crate(crate_detail, crate_metadata, dry_run)? {
                size_cleaned += crate_metadata.size();
                crate_removed += 1;
            }
        }
        Ok((size_cleaned, crate_removed))
    }
}

/// Remove crates which name is provided to delete from provided path
fn remove_crate_from_location(
    location: &Path,
    crate_detail: &CrateDetail,
    crate_metadata: &CrateMetaData,
    dry_run: bool,
) -> Result<()> {
    if location.exists() && location.is_dir() {
        for entry in fs::read_dir(location)? {
            let entry_path = entry?.path();
            if let Ok(source_url) = crate_detail.source_url_from_path(&entry_path) {
                if &Some(source_url) == crate_metadata.source() {
                    for dir_entry in fs::read_dir(entry_path)? {
                        let dir_entry_path = dir_entry?.path();
                        let crate_name = crate_metadata.name();
                        let crate_version = crate_metadata
                            .version()
                            .clone()
                            .context("failed to get crate version")?;
                        if dir_entry_path
                            .to_str()
                            .context("failed to get crate name path to str")?
                            .contains(&format!("{crate_name}-{crate_version}"))
                        {
                            delete_folder(&dir_entry_path, dry_run)?;
                        }
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

/// check if any index cache folder is empty if it is removed directory. First
/// remove all dir entry than only remove main file if it is empty
fn remove_empty_index_cache_dir(path: &Path, dry_run: bool) -> Result<()> {
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                remove_empty_index_cache_dir(&entry_path, dry_run)?;
            }
        }
        if fs::read_dir(path).map(|mut i| i.next().is_none())? {
            delete_folder(path, dry_run)?;
        }
    }
    Ok(())
}
