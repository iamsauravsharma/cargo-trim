use std::ffi::OsStr;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};
use std::{env, fs};

use anyhow::{Context as _, Result};
use owo_colors::OwoColorize as _;
use serde::{Deserialize, Serialize};

use crate::list_crate::CargoLockFiles;

/// name of the cargo target folder, resolved once since it is checked for
/// every path visited during the recursive scan
static TARGET_DIR_NAME: LazyLock<String> = LazyLock::new(|| {
    env::var("CARGO_BUILD_TARGET_DIR")
        .or_else(|_| env::var("CARGO_TARGET_DIR"))
        .unwrap_or_else(|_| String::from("target"))
});

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
    stale_days: Option<u64>,
    #[serde(skip)]
    location: PathBuf,
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

    /// point in time before which a project without activity is considered
    /// stale, `None` if stale days is not configured
    pub(crate) fn stale_cutoff(&self) -> Option<SystemTime> {
        self.stale_days
            .map(|days| SystemTime::now() - Duration::from_secs(days.saturating_mul(24 * 60 * 60)))
    }

    /// Set stale days to value
    pub(crate) fn set_stale_days(&mut self, value: u64, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} Set stale_days to {value}", "Dry run:".yellow());
        } else {
            self.stale_days = Some(value);
            if save {
                self.save()?;
            }
            println!("Set stale_days to {value}");
        }
        Ok(())
    }

    /// Unset stale days
    pub(crate) fn unset_stale_days(&mut self, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} Unset stale_days", "Dry run:".yellow());
        } else {
            self.stale_days = None;
            if save {
                self.save()?;
            }
            println!("Unset stale_days");
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
        if dry_run {
            println!(
                "{} Set scan_hidden_folder to {value:?}",
                "Dry run:".yellow(),
            );
        } else {
            self.scan_hidden_folder = value;
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
        if dry_run {
            println!(
                "{} Set scan_target_folder to {value:?}",
                "Dry run:".yellow(),
            );
        } else {
            self.scan_target_folder = value;
            if save {
                self.save()?;
            }
            println!("Set scan_target_folder to {value:?}");
        }
        Ok(())
    }

    /// add directory
    pub(crate) fn add_directory(&mut self, path: &str, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} Added {path:?}", "Dry run:".yellow());
        } else {
            self.directory.push(path.to_string());
            if save {
                self.save()?;
            }
            println!("{} {path:?}", "Added".red());
        }
        Ok(())
    }

    /// add ignore entry which is a relative or absolute path
    pub(crate) fn add_ignore(&mut self, ignore: &str, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} Added {ignore:?}", "Dry run:".yellow());
        } else {
            self.ignore.push(ignore.to_string());
            if save {
                self.save()?;
            }
            println!("{} {ignore:?}", "Added".red());
        }
        Ok(())
    }

    /// remove directory
    pub(crate) fn remove_directory(&mut self, path: &str, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} {} {path:?}", "Dry run:".yellow(), "Removed".red());
        } else {
            self.directory.retain(|data| data != path);
            if save {
                self.save()?;
            }
            println!("{} {path:?}", "Removed".red());
        }
        Ok(())
    }

    /// remove ignore entry
    pub(crate) fn remove_ignore(&mut self, ignore: &str, dry_run: bool, save: bool) -> Result<()> {
        if dry_run {
            println!("{} {} {ignore:?}", "Dry run:".yellow(), "Removed".red());
        } else {
            self.ignore.retain(|data| data != ignore);
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
        // Use symlink_metadata so we don't follow symlinks when checking existence/type
        let Ok(sym_meta) = path.symlink_metadata() else {
            return Ok(cargo_lock_files);
        };
        if sym_meta.is_symlink() {
            return Ok(cargo_lock_files);
        }
        if !self.need_to_be_ignored(path) {
            if sym_meta.is_dir() {
                for entry in fs::read_dir(path)
                    .context("failed to read directory while trying to find cargo.toml")?
                {
                    cargo_lock_files.append(self.list_cargo_locks(&entry?.path())?);
                }
            } else if sym_meta.is_file() && path.file_name() == Some(OsStr::new("Cargo.lock")) {
                let is_stale = self.stale_cutoff().is_some_and(|cutoff| {
                    path.parent()
                        .is_some_and(|dir| is_stale_project(dir, cutoff))
                });
                if is_stale {
                    println!("Ignoring stale project {}", path.display());
                } else {
                    cargo_lock_files.add_path(path.to_path_buf());
                }
            }
        }
        Ok(cargo_lock_files)
    }

    /// Collect target directories which have not been built to since `cutoff`
    pub(crate) fn stale_target_dirs(
        &self,
        path: &Path,
        cutoff: SystemTime,
        target_dirs: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let Ok(sym_meta) = path.symlink_metadata() else {
            return Ok(());
        };
        if sym_meta.is_symlink() || !sym_meta.is_dir() || self.is_ignored_path(path) {
            return Ok(());
        }
        if is_target_dir(path) {
            if !crate::utils::modified_since(path, cutoff) {
                target_dirs.push(path.to_path_buf());
            }
            // never descend into a target directory
            return Ok(());
        }
        for entry in fs::read_dir(path)
            .context("failed to read directory while trying to find target folders")?
        {
            self.stale_target_dirs(&entry?.path(), cutoff, target_dirs)?;
        }
        Ok(())
    }

    /// check if directory should be scanned for listing crates or not
    fn need_to_be_ignored(&self, path: &Path) -> bool {
        if self.is_ignored_path(path) {
            return true;
        }
        // skip target folder unless configured to be scanned
        is_target_dir(path) && !self.scan_target_folder()
    }

    /// check if path is ignored by the user configured ignore list or the
    /// hidden folder rule
    fn is_ignored_path(&self, path: &Path) -> bool {
        // match ignore entries as relative or absolute paths
        if self
            .ignore
            .iter()
            .any(|ignore| path == Path::new(ignore) || path.ends_with(ignore))
        {
            return true;
        }
        // a path without a final component cannot match the name based rules below
        let Some(file_name) = path.file_name() else {
            return false;
        };
        // a non UTF-8 name should not abort the whole scan; compare lossily
        let file_name = file_name.to_string_lossy();
        // skip hidden folder unless configured to be scanned
        file_name.starts_with('.') && !self.scan_hidden_folder()
    }

    /// save struct in the config file
    fn save(&self) -> Result<()> {
        let serialized =
            toml::to_string_pretty(&self).context("config cannot be converted to pretty toml")?;
        fs::write(&self.location, serialized).context("failed to write a value to config file")?;
        Ok(())
    }
}

/// check if path is a cargo target directory
fn is_target_dir(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy() == TARGET_DIR_NAME.as_str())
}

/// check if a project has shown no manifest changes or builds since `cutoff`
fn is_stale_project(project_dir: &Path, cutoff: SystemTime) -> bool {
    // a recent modification of the project directory itself or its manifest
    // files counts as activity
    for path in [
        project_dir.to_path_buf(),
        project_dir.join("Cargo.toml"),
        project_dir.join("Cargo.lock"),
    ] {
        if let Ok(meta) = path.symlink_metadata()
            && meta.modified().is_ok_and(|time| time > cutoff)
        {
            return false;
        }
    }
    // any recent write inside the target directory counts as a build
    !crate::utils::modified_since(&project_dir.join(TARGET_DIR_NAME.as_str()), cutoff)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

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
    fn target_folder_skipped_unless_scanned_test() {
        let target = std::env::var("CARGO_BUILD_TARGET_DIR")
            .or_else(|_| std::env::var("CARGO_TARGET_DIR"))
            .unwrap_or_else(|_| String::from("target"));
        // only meaningful when the resolved target dir is a single plain component
        if target.is_empty()
            || target.contains(std::path::MAIN_SEPARATOR)
            || target.starts_with('.')
        {
            return;
        }
        let mut path = PathBuf::from("/proj");
        path.push(&target);
        let cfg = ConfigFile {
            scan_target_folder: false,
            ..ConfigFile::default()
        };
        assert!(cfg.need_to_be_ignored(&path));
        let cfg = ConfigFile {
            scan_target_folder: true,
            ..ConfigFile::default()
        };
        assert!(!cfg.need_to_be_ignored(&path));
    }

    #[test]
    fn ignore_entry_matches_regardless_of_scan_hidden_test() {
        // a hidden folder listed in ignore is skipped even when hidden scanning is on
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
