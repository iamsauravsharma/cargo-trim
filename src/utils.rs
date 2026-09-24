use std::collections::HashSet;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;
use std::str::FromStr as _;
use std::time::SystemTime;

use anyhow::{Context as _, Result};
use owo_colors::OwoColorize as _;
use semver::Version;

/// split name and semver version part from crates full name
pub(crate) fn split_name_version(full_name: &str) -> Result<(String, Version)> {
    // only strip a trailing archive extension so a `.crate` occurring inside a
    // name is preserved
    let name = full_name.strip_suffix(".crate").unwrap_or(full_name);
    let version_split: Vec<&str> = name.split('-').collect();
    let mut version_start_position = version_split.len();
    // check a split part to check from where a semver start for crate
    for (pos, split_part) in version_split.iter().enumerate() {
        if Version::parse(split_part).is_ok() {
            version_start_position = pos;
            break;
        }
    }
    let (clear_name_vec, version_vec) = version_split.split_at(version_start_position);
    let clear_name = clear_name_vec.join("-");
    let version = Version::from_str(version_vec.join("-").as_str())
        .context("failed to parse semver version from splitted parts")?;
    Ok((clear_name, version))
}

/// delete folder with folder path provided
pub(crate) fn delete_folder(path: &Path, dry_run: bool) -> Result<()> {
    if path.exists() {
        if dry_run {
            println!(
                "{} {} {}",
                "Dry run:".yellow(),
                "Removed".red(),
                path.display()
            );
        } else if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

/// delete index .cache file
pub(crate) fn delete_index_cache(index_dir: &Path, dry_run: bool) -> Result<()> {
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
                        delete_folder(&folder_path, dry_run)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// return true if `path` or any entry inside it was modified after `cutoff`.
/// stops descending as soon as a recent entry is found
pub(crate) fn modified_since(path: &Path, cutoff: SystemTime) -> bool {
    let Ok(meta) = path.symlink_metadata() else {
        return false;
    };
    if meta.modified().is_ok_and(|time| time > cutoff) {
        return true;
    }
    if meta.is_dir()
        && let Ok(entries) = fs::read_dir(path)
    {
        for entry in entries.flatten() {
            if modified_since(&entry.path(), cutoff) {
                return true;
            }
        }
    }
    false
}

///  get size of path
pub(crate) fn get_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;
    let metadata = path.metadata();
    if let Ok(meta) = metadata {
        if meta.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path();
                total_size += get_size(&entry_path)?;
            }
        } else if meta.is_file() {
            total_size += meta.len();
        }
    }
    Ok(total_size)
}

///  get accurate bin size
pub(crate) fn get_inode_handled_size(path: &Path, inodes: &mut HashSet<u64>) -> Result<u64> {
    let mut total_size = 0;
    let metadata = path.symlink_metadata();
    if let Ok(meta) = metadata {
        if meta.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path();
                total_size += get_inode_handled_size(&entry_path, inodes)?;
            }
        } else if meta.is_file() {
            let file_size = meta.len();
            #[cfg(unix)]
            {
                let file_inode = meta.ino();
                if !inodes.contains(&file_inode) {
                    total_size += file_size;
                    inodes.insert(file_inode);
                }
            }
            #[cfg(not(unix))]
            {
                total_size += file_size;
            }
        }
    }
    Ok(total_size)
}

/// Convert size to pretty number
#[expect(
    clippy::cast_precision_loss,
    reason = "u64 to f64 precision loss is negligible for a human readable size"
)]
pub(crate) fn convert_pretty(num: u64) -> String {
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB"];
    let power_factor = if num == 0 { 0 } else { num.ilog10() / 3 };
    let pretty_bytes = format!(
        "{:7.3}",
        num as f64 / 1000_f64.powf(f64::from(power_factor))
    );
    let unit = units[power_factor as usize];
    format!("{pretty_bytes} {unit}")
}

#[cfg(test)]
mod tests {
    use std::fs::FileTimes;
    use std::time::{Duration, SystemTime};

    use semver::Version;

    use super::{convert_pretty, modified_since, split_name_version};

    #[test]
    fn modified_since_test() {
        let base = std::env::temp_dir().join(format!("cargo-trim-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("nested")).unwrap();
        let file_path = base.join("nested/file.rs");
        let file = std::fs::File::create(&file_path).unwrap();
        let old = SystemTime::now() - Duration::from_hours(60 * 24);
        let cutoff = SystemTime::now() - Duration::from_hours(30 * 24);
        // make the file and its parent directories look old
        file.set_times(FileTimes::new().set_modified(old)).unwrap();
        for dir in [&base, &base.join("nested")] {
            std::fs::File::open(dir)
                .unwrap()
                .set_times(FileTimes::new().set_modified(old))
                .unwrap();
        }
        assert!(!modified_since(&base, cutoff));
        file.set_times(FileTimes::new().set_modified(SystemTime::now()))
            .unwrap();
        assert!(modified_since(&base, cutoff));
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn split_name_version_test() {
        assert_eq!(
            split_name_version("sample_crate-0.12.0").unwrap(),
            (
                "sample_crate".to_string(),
                Version::parse("0.12.0").unwrap()
            )
        );
        assert_eq!(
            split_name_version("another-crate-name-1.4.5").unwrap(),
            (
                "another-crate-name".to_string(),
                Version::parse("1.4.5").unwrap()
            )
        );
        assert_eq!(
            split_name_version("crate-name-12-123-0.1.0").unwrap(),
            (
                "crate-name-12-123".to_string(),
                Version::parse("0.1.0").unwrap()
            )
        );
        assert_eq!(
            split_name_version("complex_name-12.0.0-rc.1").unwrap(),
            (
                "complex_name".to_string(),
                Version::parse("12.0.0-rc.1").unwrap()
            )
        );
        assert_eq!(
            split_name_version("build-number-2.3.4+was0-5").unwrap(),
            (
                "build-number".to_string(),
                Version::parse("2.3.4+was0-5").unwrap()
            )
        );
        assert_eq!(
            split_name_version("complex_spec-0.12.0-rc.1+name0.4.6").unwrap(),
            (
                "complex_spec".to_string(),
                Version::parse("0.12.0-rc.1+name0.4.6").unwrap()
            )
        );
    }

    #[test]
    fn split_name_version_strips_crate_suffix_test() {
        assert_eq!(
            split_name_version("serde-1.0.0.crate").unwrap(),
            ("serde".to_string(), Version::parse("1.0.0").unwrap())
        );
    }

    #[test]
    fn split_name_version_without_version_is_error_test() {
        assert!(split_name_version("no_version_here").is_err());
        assert!(split_name_version("also-no-version").is_err());
    }

    #[test]
    fn convert_pretty_test() {
        assert_eq!(convert_pretty(u64::MIN), "  0.000 B".to_string());
        assert_eq!(convert_pretty(12), " 12.000 B".to_string());
        assert_eq!(convert_pretty(1234), "  1.234 kB".to_string());
        assert_eq!(convert_pretty(23908), " 23.908 kB".to_string());
        assert_eq!(convert_pretty(874_940_334), "874.940 MB".to_string());
        assert_eq!(convert_pretty(8_849_909_404), "  8.850 GB".to_string());
        assert_eq!(convert_pretty(3_417_849_409_404), "  3.418 TB".to_string());
        assert_eq!(
            convert_pretty(93_453_982_182_159_417),
            " 93.454 PB".to_string()
        );
        assert_eq!(convert_pretty(u64::MAX), " 18.447 EB".to_string());
    }
}
