use std::collections::HashSet;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;
use std::str::FromStr as _;
use std::time::SystemTime;
use std::{fs, io};

use anyhow::{Context as _, Result};
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

/// check if text matches pattern where `*` in pattern matches any sequence of
/// characters, including an empty one
pub(crate) fn wildcard_match(pattern: &str, text: &str) -> bool {
    let mut parts = pattern.split('*');
    // split always yields at least one part
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    let mut parts = parts.collect::<Vec<_>>();
    let Some(last) = parts.pop() else {
        // pattern has no `*` so text must be equal to it
        return rest.is_empty();
    };
    for part in parts {
        match rest.find(part) {
            Some(position) => rest = &rest[position + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

/// check if `path` or anything inside it was modified after `cutoff`, stopping
/// as soon as one recent entry is found
pub(crate) fn modified_since(path: &Path, cutoff: SystemTime) -> bool {
    // a path which cannot be inspected counts as modified so that callers never
    // treat an unreadable project or target as stale and delete it
    let meta = match path.symlink_metadata() {
        Ok(meta) => meta,
        Err(error) => return error.kind() != io::ErrorKind::NotFound,
    };
    match meta.modified() {
        Ok(time) if time > cutoff => return true,
        Err(_) => return true,
        Ok(_) => {}
    }
    if meta.is_dir() {
        let Ok(mut entries) = fs::read_dir(path) else {
            return true;
        };
        return entries.any(|entry| entry.is_ok_and(|entry| modified_since(&entry.path(), cutoff)));
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
    use semver::Version;

    use super::{convert_pretty, split_name_version, wildcard_match};

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

    #[test]
    fn wildcard_match_test() {
        assert!(wildcard_match("abc", "abc"));
        assert!(!wildcard_match("abc", "abcd"));
        assert!(wildcard_match("a*", "abc"));
        assert!(wildcard_match("*c", "abc"));
        assert!(wildcard_match("a*c", "abc"));
        assert!(wildcard_match("a*c", "ac"));
        assert!(wildcard_match("*b*", "abc"));
        assert!(wildcard_match("a*b*c", "a_b_c"));
        assert!(!wildcard_match("a*b*c", "a_c_b"));
        assert!(!wildcard_match("ab*ba", "aba"));
        assert!(wildcard_match("*", ""));
    }
}
