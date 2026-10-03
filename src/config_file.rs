use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context as _, Result};
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};

use crate::cargo_config;
use crate::list_crate::CargoLockFiles;
use crate::utils::modified_since;

/// Stores config file information
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct ConfigFile {
    #[serde(default)]
    directory: Vec<String>,
    #[serde(default)]
    ignore: Vec<String>,
    #[serde(default)]
    scan_hidden_folder: bool,
    #[serde(default)]
    scan_target_folder: bool,
    #[serde(default)]
    stale_days: u32,
    #[serde(skip)]
    location: PathBuf,
    #[serde(skip)]
    target_dir_cache: RefCell<cargo_config::TargetDirCache>,
}

impl ConfigFile {
    /// Perform initial config file actions
    pub(crate) fn init(config_file: &Path) -> Result<Self> {
        let mut buffer = String::new();
        let mut file = fs::File::open(config_file).context("failed to open config file")?;
        file.read_to_string(&mut buffer)
            .context("failed to read config file")?;
        if buffer.is_empty() {
            let initial_config = Self::default();
            let serialize = toml::to_string_pretty(&initial_config)
                .context("failed to convert Config to string")?;
            buffer.push_str(&serialize);
        }
        let mut deserialize_config: Self =
            toml::from_str(&buffer).context("failed to convert string to Config")?;
        deserialize_config.location = config_file.to_path_buf();
        Ok(deserialize_config)
    }

    /// return vector of directory value in config file
    pub(crate) fn directory(&self) -> &Vec<String> {
        &self.directory
    }

    /// return vector of ignore values, each a relative or absolute path
    pub(crate) fn ignore(&self) -> &Vec<String> {
        &self.ignore
    }

    /// scan hidden folder
    pub(crate) fn scan_hidden_folder(&self) -> bool {
        self.scan_hidden_folder
    }

    /// scan target folder
    pub(crate) fn scan_target_folder(&self) -> bool {
        self.scan_target_folder
    }

    /// point in time before which a project without activity counts as stale,
    /// `None` when stale days is 0 and no project is ever stale
    pub(crate) fn stale_cutoff(&self) -> Option<SystemTime> {
        if self.stale_days == 0 {
            return None;
        }
        SystemTime::now().checked_sub(Duration::from_secs(
            u64::from(self.stale_days) * 24 * 60 * 60,
        ))
    }

    /// Set stale days to value, 0 turns staleness off
    pub(crate) fn set_stale_days(&mut self, value: u32, dry_run: bool, save: bool) -> Result<()> {
        if !dry_run || !save {
            self.stale_days = value;
        }
        if dry_run {
            println!("{} Set stale_days to {value}", "Dry run:".yellow());
        } else {
            if save {
                self.save()?;
            }
            println!("Set stale_days to {value}");
        }
        Ok(())
    }

    /// Set scan hidden folder to value
    pub(crate) fn set_scan_hidden_folder(
        &mut self,
        value: bool,
        dry_run: bool,
        save: bool,
    ) -> Result<()> {
        if !dry_run || !save {
            self.scan_hidden_folder = value;
        }
        if dry_run {
            println!(
                "{} Set scan_hidden_folder to {value:?}",
                "Dry run:".yellow(),
            );
        } else {
            if save {
                self.save()?;
            }
            println!("Set scan_hidden_folder to {value:?}");
        }
        Ok(())
    }

    /// Set scan target folder to value
    pub(crate) fn set_scan_target_folder(
        &mut self,
        value: bool,
        dry_run: bool,
        save: bool,
    ) -> Result<()> {
        if !dry_run || !save {
            self.scan_target_folder = value;
        }
        if dry_run {
            println!(
                "{} Set scan_target_folder to {value:?}",
                "Dry run:".yellow(),
            );
        } else {
            if save {
                self.save()?;
            }
            println!("Set scan_target_folder to {value:?}");
        }
        Ok(())
    }

    /// add directory
    pub(crate) fn add_directory(&mut self, path: &str, dry_run: bool, save: bool) -> Result<()> {
        // a value given for this run only still applies under dry run so the
        // preview reflects the requested value instead of the stored one
        if !dry_run || !save {
            self.directory.push(path.to_string());
        }
        if dry_run {
            println!("{} Added {path:?}", "Dry run:".yellow());
        } else {
            if save {
                self.save()?;
            }
            println!("{} {path:?}", "Added".red());
        }
        Ok(())
    }

    /// add ignore entry which is a relative or absolute path
    pub(crate) fn add_ignore(&mut self, ignore: &str, dry_run: bool, save: bool) -> Result<()> {
        // a value given for this run only still applies under dry run so the
        // preview reflects the requested value instead of the stored one
        if !dry_run || !save {
            self.ignore.push(ignore.to_string());
        }
        if dry_run {
            println!("{} Added {ignore:?}", "Dry run:".yellow());
        } else {
            if save {
                self.save()?;
            }
            println!("{} {ignore:?}", "Added".red());
        }
        Ok(())
    }

    /// remove directory
    pub(crate) fn remove_directory(&mut self, path: &str, dry_run: bool, save: bool) -> Result<()> {
        // a value given for this run only still applies under dry run so the
        // preview reflects the requested value instead of the stored one
        if !dry_run || !save {
            self.directory.retain(|data| data != path);
        }
        if dry_run {
            println!("{} {} {path:?}", "Dry run:".yellow(), "Removed".red());
        } else {
            if save {
                self.save()?;
            }
            println!("{} {path:?}", "Removed".red());
        }
        Ok(())
    }

    /// remove ignore entry
    pub(crate) fn remove_ignore(&mut self, ignore: &str, dry_run: bool, save: bool) -> Result<()> {
        // a value given for this run only still applies under dry run so the
        // preview reflects the requested value instead of the stored one
        if !dry_run || !save {
            self.ignore.retain(|data| data != ignore);
        }
        if dry_run {
            println!("{} {} {ignore:?}", "Dry run:".yellow(), "Removed".red());
        } else {
            if save {
                self.save()?;
            }
            println!("{} {ignore:?}", "Removed".red());
        }
        Ok(())
    }

    /// List Cargo.lock file present directories by recursively analyze all
    /// folder present in directory
    pub(crate) fn list_cargo_locks(&self, path: &Path) -> Result<CargoLockFiles> {
        let mut cargo_lock_files = CargoLockFiles::new();
        // Use symlink_metadata so we don't follow symlinks when checking
        // existence/type
        let Ok(sym_meta) = path.symlink_metadata() else {
            return Ok(cargo_lock_files);
        };
        if sym_meta.is_symlink() {
            return Ok(cargo_lock_files);
        }
        if !self.need_to_be_ignored(path) {
            if sym_meta.is_dir() {
                let skip_target = (!self.scan_target_folder() && path.join("Cargo.toml").is_file())
                    .then(|| self.resolved_target_dir(path))
                    .filter(|target| is_cargo_target_dir(target));
                for entry in fs::read_dir(path)
                    .context("failed to read directory while trying to find cargo.toml")?
                {
                    let entry_path = entry?.path();
                    if skip_target.as_ref() == Some(&entry_path) {
                        continue;
                    }
                    cargo_lock_files.append(self.list_cargo_locks(&entry_path)?);
                }
            } else if sym_meta.is_file() && path.file_name() == Some(OsStr::new("Cargo.lock")) {
                // a project is stale when nothing inside it changed since the
                // cutoff
                let stale_project = self.stale_cutoff().is_some_and(|cutoff| {
                    path.parent()
                        .is_some_and(|dir| !modified_since(dir, cutoff))
                });
                if !stale_project {
                    cargo_lock_files.add_path(path.to_path_buf());
                }
            }
        }
        Ok(cargo_lock_files)
    }

    /// target directory cargo uses for the project
    fn resolved_target_dir(&self, project_dir: &Path) -> PathBuf {
        cargo_config::target_dir(project_dir, &mut self.target_dir_cache.borrow_mut())
    }

    /// collect the target directory of every rust project below `path`, keeping
    /// only those not built to since the cutoff when one is given
    pub(crate) fn project_target_dirs(
        &self,
        path: &Path,
        only_stale_since: Option<SystemTime>,
        target_dirs: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let Ok(sym_meta) = path.symlink_metadata() else {
            return Ok(());
        };
        if sym_meta.is_symlink() || !sym_meta.is_dir() || self.need_to_be_ignored(path) {
            return Ok(());
        }
        // a directory holding a manifest is a project, so ask cargo config
        // where its build output goes instead of assuming a folder
        // named target
        let mut resolved_target = None;
        if path.join("Cargo.toml").is_file() {
            let target = self.resolved_target_dir(path);
            if is_cargo_target_dir(&target) {
                if only_stale_since.is_none_or(|cutoff| !modified_since(&target, cutoff)) {
                    target_dirs.push(target.clone());
                }
                resolved_target = Some(target);
            }
        }
        for entry in fs::read_dir(path)
            .context("failed to read directory while trying to find target folders")?
        {
            let entry_path = entry?.path();
            if resolved_target.as_ref() == Some(&entry_path) {
                continue;
            }
            self.project_target_dirs(&entry_path, only_stale_since, target_dirs)?;
        }
        Ok(())
    }

    /// check if directory should be scanned for listing crates or not, a build
    /// output folder is recognised by resolution instead of by name
    fn need_to_be_ignored(&self, path: &Path) -> bool {
        // match ignore entries as relative or absolute paths
        if self
            .ignore
            .iter()
            .any(|ignore| path == Path::new(ignore) || path.ends_with(ignore))
        {
            return true;
        }
        // a path without a final component cannot match the name based rules
        // below
        let Some(file_name) = path.file_name() else {
            return false;
        };
        // a non UTF-8 name should not abort the whole scan; compare lossily
        let file_name = file_name.to_string_lossy();
        // skip hidden folder unless configured to be scanned
        (file_name.starts_with('.') || is_os_hidden(path)) && !self.scan_hidden_folder()
    }

    /// save struct in the config file
    fn save(&self) -> Result<()> {
        let serialized =
            toml::to_string_pretty(&self).context("config cannot be converted to pretty toml")?;
        fs::write(&self.location, serialized).context("failed to write a value to config file")?;
        Ok(())
    }
}

/// cargo tags its build output, so a directory which merely shares the name or
/// the configured location is not mistaken for one
fn is_cargo_target_dir(path: &Path) -> bool {
    path.join("CACHEDIR.TAG").is_file() || path.join(".rustc_info.json").is_file()
}

/// check if the file system marks the path as hidden, which on windows is an
/// attribute rather than a leading dot
fn is_os_hidden(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;

        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        path.symlink_metadata()
            .is_ok_and(|meta| meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ConfigFile;

    fn config_with_ignore(ignore: &[&str]) -> ConfigFile {
        ConfigFile {
            ignore: ignore.iter().map(|s| (*s).to_string()).collect(),
            ..ConfigFile::default()
        }
    }

    #[test]
    fn ignore_relative_name_matches_anywhere_test() {
        let cfg = config_with_ignore(&["node_modules"]);
        assert!(cfg.need_to_be_ignored(Path::new("/a/b/node_modules")));
        assert!(cfg.need_to_be_ignored(Path::new("/x/node_modules")));
        // a whole component must match, not a substring
        assert!(!cfg.need_to_be_ignored(Path::new("/a/node_modules_old")));
    }

    #[test]
    fn ignore_relative_multi_component_matches_suffix_test() {
        let cfg = config_with_ignore(&["crates/demo"]);
        assert!(cfg.need_to_be_ignored(Path::new("/home/a/crates/demo")));
        assert!(cfg.need_to_be_ignored(Path::new("/home/b/crates/demo")));
        assert!(!cfg.need_to_be_ignored(Path::new("/home/a/crates/other")));
    }

    #[test]
    fn ignore_absolute_matches_only_exact_test() {
        let cfg = config_with_ignore(&["/abc/def"]);
        assert!(cfg.need_to_be_ignored(Path::new("/abc/def")));
        // an absolute entry must not match a deeper or relative path by suffix
        assert!(!cfg.need_to_be_ignored(Path::new("xyz/abc/def")));
        assert!(!cfg.need_to_be_ignored(Path::new("/xyz/abc/def")));
    }

    #[test]
    fn no_ignore_entry_matches_nothing_test() {
        let cfg = config_with_ignore(&[]);
        assert!(!cfg.need_to_be_ignored(Path::new("/a/b/keep_me")));
        // a path without a final component is not ignored
        assert!(!cfg.need_to_be_ignored(Path::new("/")));
    }

    #[test]
    fn hidden_folder_skipped_unless_scanned_test() {
        let hidden = Path::new("/proj/.git");
        let cfg = ConfigFile {
            scan_hidden_folder: false,
            ..ConfigFile::default()
        };
        assert!(cfg.need_to_be_ignored(hidden));
        let cfg = ConfigFile {
            scan_hidden_folder: true,
            ..ConfigFile::default()
        };
        assert!(!cfg.need_to_be_ignored(hidden));
    }

    #[test]
    fn ignore_entry_matches_regardless_of_scan_hidden_test() {
        // a hidden folder listed in ignore is skipped even when hidden scanning
        // is on
        let cfg = ConfigFile {
            ignore: vec![".cache".to_string()],
            scan_hidden_folder: true,
            ..ConfigFile::default()
        };
        assert!(cfg.need_to_be_ignored(Path::new("/a/.cache")));
        // without the ignore entry the same folder would be scanned
        let cfg = ConfigFile {
            scan_hidden_folder: true,
            ..ConfigFile::default()
        };
        assert!(!cfg.need_to_be_ignored(Path::new("/a/.cache")));
    }

    #[test]
    fn multiple_ignore_entries_test() {
        let cfg = config_with_ignore(&["node_modules", "crates/demo"]);
        assert!(cfg.need_to_be_ignored(Path::new("/x/node_modules")));
        assert!(cfg.need_to_be_ignored(Path::new("/y/crates/demo")));
        assert!(!cfg.need_to_be_ignored(Path::new("/y/crates/other")));
    }
}
