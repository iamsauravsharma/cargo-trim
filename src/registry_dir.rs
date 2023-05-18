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
        let cache_dir = cache_dir
            .to_str()
            .context("failed to convert cache dir to str")?;
        let src_dir = src_dir
            .to_str()
            .context("failed to convert src dir to str")?;
        let mut index_cache_dir = Vec::new();
        // read a index .cache dir folder for each registry by analyzing index folder
        if index_dir.exists() {
            for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
                let mut entry = entry?.path();
                entry.push(".cache");
                if entry.exists() {
                    index_cache_dir.push(
                        entry
                            .to_str()
                            .context("unable to convert index cache folder to str")?
                            .to_string(),
                    );
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
    ) -> Result<bool> {
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
                r#"{} {} "{}-{}""#,
                "Dry run:".yellow(),
                "Removed".red(),
                crate_metadata.name(),
                crate_metadata
                    .version()
                    .clone()
                    .context("failed to convert crate version")?,
            );
            Ok(true)
        } else if is_success {
            println!(
                r#"{} "{}-{}""#,
                "Removed".red(),
                crate_metadata.name(),
                crate_metadata
                    .version()
                    .clone()
                    .context("failed to convert crate version")?,
            );
            Ok(true)
        } else {
            println!(
                r#"failed to remove "{}-{}""#,
                crate_metadata.name(),
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
        list: &[CrateMetaData],
        dry_run: bool,
    ) -> Result<(u64, usize)> {
        let mut size_cleaned = 0;
        let mut crate_removed = 0;
        for crate_metadata in list {
            if self.remove_crate(crate_detail, crate_metadata, dry_run)? {
                size_cleaned += crate_metadata.size();
                crate_removed += 1;
            }
        }
        Ok((size_cleaned, crate_removed))
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
                    let crate_name = crate_metadata.name();
                    let crate_version = crate_metadata
                        .version()
                        .clone()
                        .context("failed to get crate version")?;
                    if path
                        .to_str()
                        .context("failed to get crate name path to str")?
                        .contains(&format!("{crate_name}-{crate_version}"))
                    {
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

/// check if any index cache folder is empty if it is removed directory. First
/// remove all dir entry than only remove main file if it is empty
fn remove_empty_index_cache_dir(path: &Path, dry_run: bool) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            remove_empty_index_cache_dir(&path, dry_run)?;
        }
    }
    if fs::read_dir(path).map(|mut i| i.next().is_none())? {
        delete_folder(path, dry_run)?;
    }
    Ok(())
}
