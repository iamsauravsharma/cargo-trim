use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use semver::Version;

/// split name and semver version part from crates full name
pub(crate) fn split_name_version(full_name: &str) -> Result<(String, Version)> {
    let mut name = full_name.to_string();
    name = name.replace(".crate", "");
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
        if path.is_file() {
            if dry_run {
                println!("{} {} {path:?}", "Dry run:".yellow(), "Removed".red());
            } else {
                fs::remove_file(path)?;
            }
        } else if path.is_dir() {
            if dry_run {
                println!("{} {} {path:?}", "Dry run:".yellow(), "Removed".red());
            } else {
                fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}

/// delete index .cache file
pub(crate) fn delete_index_cache(index_dir: &Path, dry_run: bool) -> Result<()> {
    for entry in fs::read_dir(index_dir)? {
        let registry_dir = entry?.path();
        for folder in fs::read_dir(registry_dir)? {
            let folder = folder?.path();
            let folder_name = folder
                .file_name()
                .context("failed to obtain index .cache file name")?;
            if folder_name == ".cache" {
                delete_folder(&folder, dry_run)?;
            }
        }
    }
    Ok(())
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
            total_size += path.metadata()?.len();
        }
    }
    Ok(total_size)
}

///  get accurate bin size
pub(crate) fn get_inode_handled_size(path: &Path, inodes: &mut Vec<u64>) -> Result<u64> {
    let mut total_size = 0;
    let metadata = path.metadata();
    if let Ok(meta) = metadata {
        if meta.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path();
                total_size += get_inode_handled_size(&entry_path, inodes)?;
            }
        } else if meta.is_file() {
            let metadata = path.metadata()?;
            let file_size = metadata.len();
            if cfg!(unix) {
                let file_inode = metadata.ino();
                if !inodes.contains(&file_inode) {
                    total_size += file_size;
                    inodes.push(file_inode);
                }
            } else {
                total_size += file_size;
            }
        }
    }
    Ok(total_size)
}

/// Convert size to pretty number
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) fn convert_pretty(num: u64) -> String {
    if num == 0 {
        return "  0.000 B".to_string();
    }
    let num = num as f64;
    let units = ["B", "kB", "MB", "GB", "TB"];
    let factor = (num.log10() / 3_f64).floor();
    let power_factor = if factor >= units.len() as f64 {
        (units.len() - 1) as f64
    } else {
        factor
    };
    let pretty_bytes = format!("{:7.3}", num / 1000_f64.powf(power_factor));
    let unit = units[power_factor as usize];
    format!("{pretty_bytes} {unit}")
}

#[cfg(test)]
mod test {
    use semver::Version;

    use super::{convert_pretty, split_name_version};

    #[test]
    fn test_split_name_version() {
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
    fn test_convert_pretty() {
        assert_eq!(convert_pretty(0), "  0.000 B".to_string());
        assert_eq!(convert_pretty(12), " 12.000 B".to_string());
        assert_eq!(convert_pretty(1234), "  1.234 kB".to_string());
        assert_eq!(convert_pretty(23908), " 23.908 kB".to_string());
        assert_eq!(convert_pretty(874_940_334), "874.940 MB".to_string());
        assert_eq!(convert_pretty(8_849_909_404), "  8.850 GB".to_string());
        assert_eq!(convert_pretty(3_417_849_409_404), "  3.418 TB".to_string());
        assert_eq!(
            convert_pretty(93_453_982_182_159_417),
            "93453.982 TB".to_string()
        );
    }
}
