use std::{fs, path::Path};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use crate::{
    crate_detail::CrateDetail,
    utils::{clear_version_value, delete_folder},
};

// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir<'a> {
    cache_dir: &'a str,
    src_dir: &'a str,
    index_cache_dir: Vec<String>,
    installed_crate: Vec<String>,
}

impl<'a> RegistryDir<'a> {
    // Create new RegistryDir
    pub(crate) fn new(
        cache_dir: &'a Path,
        src_dir: &'a Path,
        index_dir: &Path,
        installed_crate: &[String],
    ) -> Result<Self> {
        let cache_dir = cache_dir.to_str().unwrap();
        let src_dir = src_dir.to_str().unwrap();
        let mut index_cache_dir = Vec::new();
        // read a index .cache dir folder for each registry by analyzing index folder
        for entry in fs::read_dir(index_dir).context("failed to read index directory")? {
            let entry = entry?.path();
            let registry_dir = entry.as_path();
            for folder in fs::read_dir(registry_dir).context("failed to read registry directory")? {
                let folder = folder?.path();
                let folder_name = folder
                    .file_name()
                    .context("failed to get file name form registry sub directory")?;
                if folder_name == ".cache" {
                    index_cache_dir.push(folder.to_str().unwrap().to_string());
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

    // Remove crate from src & cache directory
    pub(crate) fn remove_crate(&mut self, crate_name: &str, dry_run: bool) {
        let mut is_success;
        // remove crate from cache dir
        is_success = remove_crate(Path::new(&self.cache_dir), crate_name, dry_run).is_ok();
        // remove crate from index dir
        is_success =
            remove_crate(Path::new(&self.src_dir), crate_name, dry_run).is_ok() && is_success;
        let split_value = clear_version_value(crate_name);
        let name = split_value.0;
        let index_cache = self.index_cache_dir.clone();
        // remove index cache dir if their is only one crate. It will also clean crate
        // name from installed crate name owned locally by it so when two version of
        // same crate is deleted it properly remove index cache
        for index_cache_dir in &index_cache {
            let same_name_list = self.installed_crate.iter().filter(|&x| x.contains(&name));
            if same_name_list.count() == 1 {
                is_success = remove_index_cache(Path::new(&index_cache_dir), crate_name, dry_run)
                    .is_ok()
                    && is_success;
            }
            is_success = remove_empty_index_cache_dir(Path::new(&index_cache_dir), dry_run).is_ok()
                && is_success;
            self.installed_crate.retain(|x| x != crate_name);
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
            println!(
                "Partially failed to remove some directory and file of {:?}",
                crate_name
            );
        }
    }

    // Remove list of crates
    pub(crate) fn remove_crate_list(
        &mut self,
        crate_detail: &CrateDetail,
        list: &[String],
        dry_run: bool,
    ) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name, dry_run);
            size_cleaned += crate_detail.find(crate_name, "REGISTRY");
        }
        size_cleaned
    }
}

// Remove crates which name is provided to delete
fn remove_crate(path: &Path, value: &str, dry_run: bool) -> Result<()> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.to_str().unwrap().contains(value) {
                    delete_folder(&path, dry_run)?;
                }
            }
        }
    }
    Ok(())
}

// determine crate index cache location and remove crate index cache
fn remove_index_cache(path: &Path, crate_name: &str, dry_run: bool) -> Result<()> {
    let mut crate_index_cache_location = path.to_path_buf();
    let split_value = clear_version_value(crate_name);
    let name = split_value.0;
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

// check if any index cache folder is empty if it is it is removed out
fn remove_empty_index_cache_dir(path: &Path, dry_run: bool) -> Result<()> {
    if path
        .read_dir()
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
    {
        delete_folder(path, dry_run)?;
    } else {
        for entry in path.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                remove_empty_index_cache_dir(path.as_path(), dry_run)?;
            }
        }
    }
    Ok(())
}
