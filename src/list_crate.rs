use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use anyhow::{Context as _, Result};
use semver::Version;

use crate::config_file::ConfigFile;
use crate::dir_path::DirPath;
use crate::filter::CrateFilter;
use crate::installed::{self, CrateMetaData, Sources};
use crate::lock_file::{self, CargoLockFiles};

/// state of an installed crate, a crate without any state is only installed
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CrateState {
    /// older version of a crate which also has a newer version, or git
    /// checkout of a revision which is not the latest fetched one
    Old,
    /// crate not used by any Cargo.lock file of rust projects
    Orphan,
    /// crate protected from cleaning by filter
    Kept,
}

/// crates to select out of crate list
#[derive(Clone, Copy)]
pub(crate) enum Selection {
    /// every installed crate, including kept ones
    Installed,
    /// every crate which can be cleaned
    All,
    /// old crates which can be cleaned
    Old,
    /// orphan crates which can be cleaned
    Orphan,
    /// crates which are both old and orphan and can be cleaned
    OldOrphan,
    /// crates protected from cleaning by filter
    Kept,
}

/// installed crate along with every state it is in
struct ListedCrate {
    metadata: CrateMetaData,
    states: Vec<CrateState>,
}

impl ListedCrate {
    fn new(metadata: CrateMetaData, old: bool, orphan: bool, filter: &CrateFilter) -> Self {
        let states = [
            (old, CrateState::Old),
            (orphan, CrateState::Orphan),
            (!filter.allows(&metadata), CrateState::Kept),
        ]
        .into_iter()
        .filter_map(|(present, state)| present.then_some(state))
        .collect();
        Self { metadata, states }
    }

    fn is(&self, state: CrateState) -> bool {
        self.states.contains(&state)
    }

    /// check if crate is in states asked by selection, ignoring filter
    fn has_states_of(&self, selection: Selection) -> bool {
        match selection {
            Selection::Installed | Selection::All | Selection::Kept => true,
            Selection::Old => self.is(CrateState::Old),
            Selection::Orphan => self.is(CrateState::Orphan),
            Selection::OldOrphan => self.is(CrateState::Old) && self.is(CrateState::Orphan),
        }
    }

    fn is_selected(&self, selection: Selection) -> bool {
        match selection {
            Selection::Installed => true,
            Selection::Kept => self.is(CrateState::Kept),
            _ => !self.is(CrateState::Kept) && self.has_states_of(selection),
        }
    }
}

/// struct to store all installed crates with their state
pub(crate) struct CrateList {
    bin: Vec<CrateMetaData>,
    registry: Vec<ListedCrate>,
    git: Vec<ListedCrate>,
    cargo_lock_files: CargoLockFiles,
}

impl CrateList {
    /// create list of all installed crates along with their state
    pub(crate) fn create_list(
        dir_path: &DirPath,
        config_file: &ConfigFile,
        sources: &Sources,
    ) -> Result<Self> {
        let (cargo_lock_files, used_registry, used_git) =
            lock_file::used_crates(config_file, sources)?;
        Ok(Self {
            bin: installed::installed_bin(dir_path.bin_dir())?,
            registry: list_registry(
                installed::installed_registry(dir_path.src_dir(), dir_path.cache_dir())?,
                &used_registry,
                config_file.filter(),
            ),
            git: list_git(
                installed::installed_git(dir_path.checkout_dir(), dir_path.db_dir())?,
                &used_git,
                &latest_revs(dir_path.db_dir())?,
                config_file.filter(),
            ),
            cargo_lock_files,
        })
    }

    /// provide list of installed bin
    pub(crate) fn bin(&self) -> &[CrateMetaData] {
        &self.bin
    }

    /// provide registry crates matching selection
    pub(crate) fn registry(&self, selection: Selection) -> Vec<CrateMetaData> {
        select(&self.registry, selection)
    }

    /// provide git crates matching selection
    pub(crate) fn git(&self, selection: Selection) -> Vec<CrateMetaData> {
        select(&self.git, selection)
    }

    /// number of registry crates in states asked by selection which filter
    /// keeps from cleaning
    pub(crate) fn registry_kept(&self, selection: Selection) -> usize {
        kept_count(&self.registry, selection)
    }

    /// number of git crates in states asked by selection which filter keeps
    /// from cleaning
    pub(crate) fn git_kept(&self, selection: Selection) -> usize {
        kept_count(&self.git, selection)
    }

    /// List Cargo.lock file
    pub(crate) fn cargo_lock_files(&self) -> &CargoLockFiles {
        &self.cargo_lock_files
    }
}

/// count crates in states asked by selection which are kept by filter
fn kept_count(crates: &[ListedCrate], selection: Selection) -> usize {
    crates
        .iter()
        .filter(|listed| listed.is(CrateState::Kept) && listed.has_states_of(selection))
        .count()
}

/// clone metadata of crates matching selection
fn select(crates: &[ListedCrate], selection: Selection) -> Vec<CrateMetaData> {
    crates
        .iter()
        .filter(|listed| listed.is_selected(selection))
        .map(|listed| listed.metadata.clone())
        .collect()
}

/// registry crates with state. A crate is old when the same registry holds a
/// newer version of it and orphan when no Cargo.lock file uses it
fn list_registry(
    installed: Vec<CrateMetaData>,
    used: &[CrateMetaData],
    filter: &CrateFilter,
) -> Vec<ListedCrate> {
    let mut newest: HashMap<(String, Option<String>), Version> = HashMap::new();
    for crate_metadata in &installed {
        if let Some(version) = crate_metadata.version() {
            let key = (
                crate_metadata.name().clone(),
                crate_metadata.source().cloned(),
            );
            if newest.get(&key).is_none_or(|newest| version > newest) {
                newest.insert(key, version.clone());
            }
        }
    }
    installed
        .into_iter()
        .map(|crate_metadata| {
            let key = (
                crate_metadata.name().clone(),
                crate_metadata.source().cloned(),
            );
            let old = crate_metadata
                .version()
                .is_some_and(|version| newest.get(&key).is_some_and(|newest| version < newest));
            // used list is sorted so binary search can be used
            let orphan = used.binary_search(&crate_metadata).is_err();
            ListedCrate::new(crate_metadata, old, orphan, filter)
        })
        .collect()
}

/// git crates with state. A checkout is old when its revision is not the latest
/// fetched one of its repository while a git db is never old. A checkout is
/// orphan when no Cargo.lock file uses its revision and a git db when no
/// Cargo.lock file uses its repository
fn list_git(
    installed: Vec<CrateMetaData>,
    used: &[CrateMetaData],
    latest_revs: &HashMap<String, String>,
    filter: &CrateFilter,
) -> Vec<ListedCrate> {
    installed
        .into_iter()
        .map(|crate_metadata| {
            // checkout is named repo-rev while git db is named repo-HEAD
            let rev = crate_metadata
                .name()
                .rsplit_once('-')
                .map_or("", |(_, rev)| rev);
            let is_db = rev == "HEAD";
            // checkout folder is an abbreviation of the full latest hash when
            // it is the latest revision. A checkout of a repository whose
            // latest revision is unknown is not old so it is never
            // removed by mistake
            let old = !is_db
                && crate_metadata
                    .source()
                    .and_then(|source| latest_revs.get(source))
                    .is_some_and(|latest| !latest.starts_with(rev));
            let orphan = if is_db {
                !used
                    .iter()
                    .any(|used| used.source() == crate_metadata.source())
            } else {
                // used list is sorted so binary search can be used
                used.binary_search(&crate_metadata).is_err()
            };
            ListedCrate::new(crate_metadata, old, orphan, filter)
        })
        .collect()
}

/// full hash of latest fetched commit of every git db, keyed by db folder name
/// which is also the source of its checkouts
fn latest_revs(db_dir: &Path) -> Result<HashMap<String, String>> {
    let mut revs = HashMap::new();
    if !db_dir.is_dir() {
        return Ok(revs);
    }
    for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_dir()
            && let Some(rev) = latest_rev_value(&path)?
        {
            revs.insert(name.to_string(), rev);
        }
    }
    Ok(revs)
}

/// get full hash of latest fetched commit from git repository, `None` if the
/// repository has no usable `FETCH_HEAD`
fn latest_rev_value(path: &Path) -> Result<Option<String>> {
    let fetch_head_file = path.join("FETCH_HEAD");
    let content = match fs::read_to_string(&fetch_head_file) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read {}", fetch_head_file.display()));
        }
    };
    // first word is the full hash of the latest fetched commit
    Ok(content.split_whitespace().next().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use semver::Version;

    use super::{CrateState, ListedCrate, Selection, list_git, list_registry};
    use crate::filter::CrateFilter;
    use crate::installed::CrateMetaData;

    fn registry(name: &str, version: &str, source: &str) -> CrateMetaData {
        CrateMetaData::new(
            name.to_string(),
            Some(Version::parse(version).unwrap()),
            Some(source.to_string()),
        )
    }

    fn git(name: &str, source: &str) -> CrateMetaData {
        CrateMetaData::new(name.to_string(), None, Some(source.to_string()))
    }

    fn states(listed: &[ListedCrate]) -> Vec<(bool, bool)> {
        listed
            .iter()
            .map(|listed| (listed.is(CrateState::Old), listed.is(CrateState::Orphan)))
            .collect()
    }

    #[test]
    fn old_registry_crate_is_per_registry_test() {
        // sorted order puts serde 1.0.0 of registry b between the two versions
        // of registry a
        let mut installed = vec![
            registry("serde", "2.0.0", "a"),
            registry("serde", "1.0.0", "b"),
            registry("serde", "1.0.0", "a"),
            registry("tokio", "1.0.0", "a"),
        ];
        installed.sort();
        let used = vec![registry("serde", "2.0.0", "a")];
        assert_eq!(
            states(&list_registry(installed, &used, &CrateFilter::default())),
            [(true, true), (false, true), (false, false), (false, true)]
        );
    }

    #[test]
    fn old_registry_crate_with_several_newer_versions_test() {
        let installed = vec![
            registry("syn", "1.0.0", "a"),
            registry("syn", "1.0.1", "a"),
            registry("syn", "2.0.0", "a"),
        ];
        assert_eq!(
            states(&list_registry(installed, &[], &CrateFilter::default())),
            [(true, true), (true, true), (false, true)]
        );
    }

    #[test]
    fn latest_git_checkout_is_not_old_test() {
        let installed = vec![
            git("repo-HEAD", "repo-1a2b3c"),
            git("repo-abcdef1", "repo-1a2b3c"),
            git("repo-1234567", "repo-1a2b3c"),
            git("repo-abcdef12", "repo-1a2b3c"),
            git("repo-abcdef9", "repo-1a2b3c"),
            git("other-7654321", "other-9f8e7d"),
        ];
        let latest_revs = HashMap::from([(
            "repo-1a2b3c".to_string(),
            "abcdef1234567890abcdef1234567890abcdef12".to_string(),
        )]);
        let used = vec![git("repo-abcdef1", "repo-1a2b3c")];
        // git db is never old and is used through its checkout, latest checkout
        // is not old whatever length its abbreviation has and a checkout of
        // unknown latest revision is not old either
        assert_eq!(
            states(&list_git(
                installed,
                &used,
                &latest_revs,
                &CrateFilter::default()
            )),
            [
                (false, false),
                (false, false),
                (true, true),
                (false, true),
                (true, true),
                (false, true),
            ]
        );
    }

    #[test]
    fn kept_crate_is_never_selected_for_cleaning_test() {
        let installed = vec![
            registry("serde", "1.0.0", "a"),
            registry("serde", "2.0.0", "a"),
        ];
        let filter = CrateFilter::try_from(vec!["!serde@1".to_string()]).unwrap();
        let listed = list_registry(installed, &[], &filter);
        let selected = |selection| {
            listed
                .iter()
                .filter(|listed| listed.is_selected(selection))
                .count()
        };
        assert_eq!(selected(Selection::Installed), 2);
        assert_eq!(selected(Selection::All), 1);
        assert_eq!(selected(Selection::Kept), 1);
        // serde 1.0.0 is old and orphan but kept
        assert_eq!(selected(Selection::Old), 0);
        assert_eq!(selected(Selection::OldOrphan), 0);
        assert_eq!(selected(Selection::Orphan), 1);
        let kept = |selection| super::kept_count(&listed, selection);
        assert_eq!(kept(Selection::All), 1);
        assert_eq!(kept(Selection::Old), 1);
        assert_eq!(kept(Selection::OldOrphan), 1);
        assert_eq!(kept(Selection::Orphan), 1);
    }
}
