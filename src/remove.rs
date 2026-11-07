use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context as _, Result};

use crate::color::Paint as _;
use crate::installed::CrateMetaData;
use crate::output::{dry_run_prefix, info, is_dry_run, verbose};

/// delete file or folder, nothing is deleted under dry run, returns whether
/// path existed
fn remove(path: &Path) -> io::Result<bool> {
    let meta = match path.symlink_metadata() {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
        Ok(meta) => meta,
    };
    if !is_dry_run() {
        if meta.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(true)
}

/// delete file or folder which is an item of its own and print it as progress
/// line, a missing path counts as deleted
pub(crate) fn delete_folder(path: &Path) -> Result<()> {
    if remove(path).with_context(|| format!("failed to remove {}", path.display()))? {
        info!("{} {}", "Removed".red(), path.display());
    }
    Ok(())
}

/// delete file or folder which is part of an item printing its own line, so
/// path is printed only with verbose output, a missing path counts as deleted
pub(crate) fn remove_path(path: &Path) -> io::Result<()> {
    match remove(path) {
        Ok(true) => verbose!("{}Removed {}", dry_run_prefix(), path.display()),
        Ok(false) => {}
        Err(error) => {
            verbose!("Failed to remove {}: {error}", path.display());
            return Err(error);
        }
    }
    Ok(())
}

/// delete every path of crate along with extra paths and print a single line
/// about it, returns whether everything was deleted
pub(crate) fn remove_crate(crate_metadata: &CrateMetaData, extra_paths: &[PathBuf]) -> bool {
    // attempt every deletion so an earlier failure does not skip the rest
    let mut removed = true;
    for path in crate_metadata.paths().iter().chain(extra_paths) {
        removed = remove_path(path).is_ok() && removed;
    }
    if removed {
        info!("{} {crate_metadata}", "Removed".red());
    } else {
        println!("Failed to remove {crate_metadata}");
    }
    removed
}

/// delete index .cache file
pub(crate) fn delete_index_cache(index_dir: &Path) -> Result<()> {
    if index_dir.exists() && index_dir.is_dir() {
        for entry in fs::read_dir(index_dir)? {
            let registry_dir = entry?.path();
            if registry_dir.is_dir() {
                let mut config_file = registry_dir.clone();
                config_file.push("config.json");
                if config_file.exists() {
                    continue;
                }
                for folder in fs::read_dir(registry_dir)? {
                    let folder_path = folder?.path();
                    let folder_name = folder_path
                        .file_name()
                        .context("failed to obtain index .cache file name")?;
                    if folder_name == ".cache" {
                        delete_folder(&folder_path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Remove list of git crates, returns total size and number of removed crates
pub(crate) fn remove_git_crates(list: &[CrateMetaData]) -> (u64, usize) {
    let mut size_cleaned = 0;
    let mut crate_removed = 0;
    for crate_metadata in list {
        if remove_crate(crate_metadata, &[]) {
            size_cleaned += crate_metadata.size();
            crate_removed += 1;
        }
    }
    (size_cleaned, crate_removed)
}

/// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir {
    index_cache_dir: Vec<PathBuf>,
    installed_crate: Vec<CrateMetaData>,
}

impl RegistryDir {
    /// Create new registry dir
    pub(crate) fn new(index_dir: &Path, installed_crate: &[CrateMetaData]) -> Result<Self> {
        let mut index_cache_dir = Vec::new();
        // read a index .cache dir folder for each registry by analyzing index
        // folder
        if index_dir.exists() && index_dir.is_dir() {
            for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
                let mut entry_path = entry?.path();
                entry_path.push(".cache");
                if entry_path.exists() {
                    index_cache_dir.push(entry_path);
                }
            }
        }

        Ok(Self {
            index_cache_dir,
            installed_crate: installed_crate.to_owned(),
        })
    }

    /// Remove crate from src & cache directory, along with its index cache
    /// once no other version of it is left in the registry
    fn remove_crate(&mut self, crate_metadata: &CrateMetaData) -> Result<bool> {
        let mut index_cache_paths = Vec::new();
        for index in &self.index_cache_dir {
            let index_name = index
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                .context("failed to get index parent")?;
            if crate_metadata.source().map(String::as_str) != Some(index_name) {
                continue;
            }
            let same_name_count = self
                .installed_crate
                .iter()
                .filter(|installed| {
                    installed.name() == crate_metadata.name()
                        && installed.source() == crate_metadata.source()
                })
                .count();
            if same_name_count == 1 {
                index_cache_paths.push(index_cache_path(index, crate_metadata.name())?);
            }
            self.installed_crate
                .retain(|installed| installed != crate_metadata);
        }
        Ok(remove_crate(crate_metadata, &index_cache_paths))
    }

    /// Remove list of crates, returns total size and number of removed crates
    pub(crate) fn remove_crate_list(
        &mut self,
        crate_metadata_list: &[CrateMetaData],
    ) -> Result<(u64, usize)> {
        let mut size_cleaned = 0;
        let mut crate_removed = 0;
        for crate_metadata in crate_metadata_list {
            if self.remove_crate(crate_metadata)? {
                size_cleaned += crate_metadata.size();
                crate_removed += 1;
            }
        }
        for index in &self.index_cache_dir {
            remove_empty_index_cache_dir(index)?;
        }
        Ok((size_cleaned, crate_removed))
    }
}

/// location of crate index cache inside index cache folder
fn index_cache_path(index_cache_dir: &Path, name: &str) -> Result<PathBuf> {
    let mut path = index_cache_dir.to_path_buf();
    // slice with `get` so a multi-byte crate name errors instead of panicking
    let invalid = || format!("crate name {name:?} is not valid for index cache slicing");
    match name.len() {
        1 => path.push("1"),
        2 => path.push("2"),
        3 => {
            path.push("3");
            path.push(name.get(..1).with_context(invalid)?);
        }
        _ => {
            path.push(name.get(..2).with_context(invalid)?);
            path.push(name.get(2..4).with_context(invalid)?);
        }
    }
    path.push(name);
    Ok(path)
}

/// check if any index cache folder is empty if it is removed directory. First
/// remove all dir entry than only remove main file if it is empty
fn remove_empty_index_cache_dir(path: &Path) -> Result<()> {
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                remove_empty_index_cache_dir(&entry_path)?;
            }
        }
        if fs::read_dir(path).map(|mut i| i.next().is_none())? {
            remove_path(path)?;
        }
    }
    Ok(())
}
