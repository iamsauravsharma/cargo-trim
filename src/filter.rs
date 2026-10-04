use anyhow::{Context as _, Result, bail};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::installed::CrateMetaData;
use crate::utils::wildcard_match;

/// version part of a package spec, a semver requirement for registry crate or
/// a revision prefix for git crate
#[derive(Clone)]
struct VersionPattern {
    text: String,
    requirement: Option<VersionReq>,
}

impl VersionPattern {
    fn parse(text: &str) -> Result<Self> {
        if text.is_empty() {
            bail!("version after `@` is empty");
        }
        // a full version means exactly that version like cargo `-p name@1.0.0`
        // while anything else is read as a semver requirement
        let requirement = if Version::parse(text).is_ok() {
            VersionReq::parse(&format!("={text}")).ok()
        } else {
            VersionReq::parse(text).ok()
        };
        if requirement.is_none() && !text.chars().all(|c| c.is_ascii_alphanumeric()) {
            bail!("{text:?} is neither a semver version requirement nor a git revision");
        }
        Ok(Self {
            text: text.to_string(),
            requirement,
        })
    }

    fn matches_version(&self, version: &Version) -> bool {
        self.requirement
            .as_ref()
            .is_some_and(|requirement| requirement.matches(version))
    }

    fn matches_rev(&self, rev: &str) -> bool {
        rev.starts_with(self.text.as_str())
    }
}

/// cargo style package spec `[registry:]name[@version]`. Registry and name may
/// contain `*` wildcard, registry is matched against the source folder name
/// with or without its hash suffix and version is a semver requirement for
/// registry crate or a revision prefix for git crate
#[derive(Clone)]
pub(crate) struct PackageSpec {
    registry: Option<String>,
    name: String,
    version: Option<VersionPattern>,
}

impl PackageSpec {
    pub(crate) fn parse(spec: &str) -> Result<Self> {
        // crate name never contains `:` so first `:` separates the registry
        let (registry, rest) = match spec.split_once(':') {
            Some(("", _)) => bail!("registry is empty in {spec:?}"),
            Some((registry, rest)) => (Some(registry.to_string()), rest),
            None => (None, spec),
        };
        let (name, version) = match rest.split_once('@') {
            Some((name, version)) => (name, Some(VersionPattern::parse(version)?)),
            None => (rest, None),
        };
        if name.is_empty() {
            bail!("crate name is empty in {spec:?}");
        }
        Ok(Self {
            registry,
            name: name.to_string(),
            version,
        })
    }

    fn matches(&self, crate_metadata: &CrateMetaData) -> bool {
        if let Some(registry) = &self.registry
            && !crate_metadata
                .source()
                .is_some_and(|source| registry_matches(registry, source))
        {
            return false;
        }
        let name = crate_metadata.name();
        if let Some(version) = crate_metadata.version() {
            return wildcard_match(&self.name, name)
                && self
                    .version
                    .as_ref()
                    .is_none_or(|pattern| pattern.matches_version(version));
        }
        // git crate name is in form of repo-rev where rev is a short commit
        // hash or HEAD for git db
        let (base_name, rev) = name
            .rsplit_once('-')
            .map_or((name.as_str(), None), |(base, rev)| (base, Some(rev)));
        wildcard_match(&self.name, base_name)
            && self
                .version
                .as_ref()
                .is_none_or(|pattern| rev.is_some_and(|rev| pattern.matches_rev(rev)))
    }
}

/// check if registry pattern matches source folder name such as
/// `index.crates.io-1949cf8c6b5b557f`, either in full or without its hash
/// suffix so `index.crates.io` matches it too
fn registry_matches(pattern: &str, source: &str) -> bool {
    wildcard_match(pattern, source)
        || source
            .rsplit_once('-')
            .is_some_and(|(without_hash, _)| wildcard_match(pattern, without_hash))
}

/// decide which crates can be cleaned from a list of package spec stored as
/// plain strings in config file. When any spec without `!` is present only
/// crates matching one of them are cleaned, and a crate matching a spec
/// starting with `!` is never cleaned
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub(crate) struct CrateFilter {
    raw: Vec<String>,
    include: Vec<PackageSpec>,
    exclude: Vec<PackageSpec>,
}

impl TryFrom<Vec<String>> for CrateFilter {
    type Error = anyhow::Error;

    fn try_from(entries: Vec<String>) -> Result<Self> {
        let mut filter = Self::default();
        for entry in &entries {
            filter.add(entry)?;
        }
        Ok(filter)
    }
}

impl From<CrateFilter> for Vec<String> {
    fn from(filter: CrateFilter) -> Self {
        filter.raw
    }
}

impl CrateFilter {
    /// add an entry, failing if it is not a valid package spec
    pub(crate) fn add(&mut self, entry: &str) -> Result<()> {
        let invalid = || format!("invalid filter {entry:?}");
        if let Some(spec) = entry.strip_prefix('!') {
            self.exclude
                .push(PackageSpec::parse(spec).with_context(invalid)?);
        } else {
            self.include
                .push(PackageSpec::parse(entry).with_context(invalid)?);
        }
        self.raw.push(entry.to_string());
        Ok(())
    }

    /// remove all occurrence of an entry
    pub(crate) fn remove(&mut self, entry: &str) {
        let entries = std::mem::take(&mut self.raw);
        *self = Self::default();
        for kept in entries.iter().filter(|kept| *kept != entry) {
            // entries were valid when added so adding them again cannot fail
            let _ = self.add(kept);
        }
    }

    /// check if crate can be cleaned
    pub(crate) fn allows(&self, crate_metadata: &CrateMetaData) -> bool {
        let matches = |specs: &[PackageSpec]| specs.iter().any(|spec| spec.matches(crate_metadata));
        (self.include.is_empty() || matches(&self.include)) && !matches(&self.exclude)
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::{CrateFilter, PackageSpec};
    use crate::installed::CrateMetaData;

    fn filter(specs: &[&str]) -> CrateFilter {
        CrateFilter::try_from(
            specs
                .iter()
                .map(|spec| (*spec).to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn registry(name: &str, version: &str) -> CrateMetaData {
        CrateMetaData::new(
            name.to_string(),
            Some(Version::parse(version).unwrap()),
            None,
        )
    }

    fn git(name: &str) -> CrateMetaData {
        CrateMetaData::new(name.to_string(), None, None)
    }

    fn matches(spec: &str, crate_metadata: &CrateMetaData) -> bool {
        PackageSpec::parse(spec).unwrap().matches(crate_metadata)
    }

    fn from_registry(name: &str, version: &str, source: &str) -> CrateMetaData {
        CrateMetaData::new(
            name.to_string(),
            Some(Version::parse(version).unwrap()),
            Some(source.to_string()),
        )
    }

    #[test]
    fn registry_spec_test() {
        let crates_io = from_registry("serde", "1.0.0", "index.crates.io-1949cf8c6b5b557f");
        let company = from_registry("serde", "1.0.0", "my-company.com-0123456789abcdef");
        // full folder name or folder name without hash suffix
        assert!(matches(
            "index.crates.io-1949cf8c6b5b557f:serde",
            &crates_io
        ));
        assert!(matches("index.crates.io:serde", &crates_io));
        assert!(!matches("index.crates.io:serde", &company));
        // wildcard in registry and name
        assert!(matches("*company*:*", &company));
        assert!(!matches("*company*:*", &crates_io));
        assert!(matches("my-company.com:ser*@^1", &company));
        assert!(!matches("my-company.com:serde@2", &company));
        // crate without source such as installed binary never matches registry
        assert!(!matches("*:serde", &registry("serde", "1.0.0")));
        // spec without registry matches every registry
        assert!(matches("serde", &company));
    }

    #[test]
    fn registry_filter_keeps_whole_registry_test() {
        let filter = filter(&["!my-company.com:*"]);
        assert!(!filter.allows(&from_registry("serde", "1.0.0", "my-company.com-0123")));
        assert!(filter.allows(&from_registry("serde", "1.0.0", "index.crates.io-1949")));
    }

    #[test]
    fn exact_name_test() {
        assert!(matches("serde", &registry("serde", "1.0.0")));
        assert!(!matches("serde", &registry("serde_json", "1.0.0")));
    }

    #[test]
    fn wildcard_name_test() {
        assert!(matches("windows-*", &registry("windows-sys", "0.52.0")));
        assert!(!matches("windows-*", &registry("windows", "0.52.0")));
        assert!(matches("*-sys", &registry("openssl-sys", "0.9.0")));
        assert!(matches("*serde*", &registry("my_serde_ext", "0.1.0")));
        assert!(matches("*", &registry("anything", "0.1.0")));
        assert!(matches("ser*de", &registry("serde", "1.0.0")));
        assert!(matches("ser*de", &registry("ser_fast_de", "1.0.0")));
        assert!(!matches("ser*de", &registry("serde_json", "1.0.0")));
    }

    #[test]
    fn full_version_is_exact_test() {
        assert!(matches("syn@1.0.109", &registry("syn", "1.0.109")));
        assert!(!matches("syn@1.0.109", &registry("syn", "1.0.110")));
    }

    #[test]
    fn version_requirement_test() {
        assert!(matches("tokio@^1", &registry("tokio", "1.30.0")));
        assert!(!matches("tokio@^1", &registry("tokio", "0.2.0")));
        assert!(matches("syn@<2", &registry("syn", "1.0.109")));
        assert!(!matches("syn@<2", &registry("syn", "2.0.0")));
        assert!(matches("syn@1", &registry("syn", "1.0.0")));
    }

    #[test]
    fn git_crate_test() {
        assert!(matches("my-repo", &git("my-repo-abcdef1")));
        assert!(matches("my-repo", &git("my-repo-HEAD")));
        assert!(matches("my-repo@abc", &git("my-repo-abcdef1")));
        assert!(!matches("my-repo@abc", &git("my-repo-1234567")));
        assert!(matches("my-*", &git("my-repo-abcdef1")));
        assert!(!matches("my-repo@^1", &git("my-repo-abcdef1")));
    }

    #[test]
    fn invalid_spec_test() {
        assert!(PackageSpec::parse("").is_err());
        assert!(PackageSpec::parse("@1.0.0").is_err());
        assert!(PackageSpec::parse("serde@").is_err());
        assert!(PackageSpec::parse("serde@>>1").is_err());
        assert!(PackageSpec::parse(":serde").is_err());
        assert!(PackageSpec::parse("registry:").is_err());
        assert!(CrateFilter::try_from(vec!["serde@".to_string()]).is_err());
        assert!(CrateFilter::try_from(vec!["!".to_string()]).is_err());
    }

    #[test]
    fn exclude_never_cleaned_test() {
        let filter = filter(&["!serde", "!windows-*"]);
        assert!(!filter.allows(&registry("serde", "1.0.0")));
        assert!(!filter.allows(&registry("windows-sys", "0.52.0")));
        assert!(filter.allows(&registry("tokio", "1.0.0")));
    }

    #[test]
    fn include_restricts_cleaning_test() {
        let filter = filter(&["tokio*"]);
        assert!(filter.allows(&registry("tokio-util", "0.7.0")));
        assert!(!filter.allows(&registry("serde", "1.0.0")));
    }

    #[test]
    fn exclude_takes_precedence_over_include_test() {
        let filter = filter(&["tokio*", "!tokio"]);
        assert!(!filter.allows(&registry("tokio", "1.0.0")));
        assert!(filter.allows(&registry("tokio-util", "0.7.0")));
        assert!(!filter.allows(&registry("serde", "1.0.0")));
    }

    #[test]
    fn empty_filter_allows_everything_test() {
        let filter = filter(&[]);
        assert!(filter.allows(&registry("serde", "1.0.0")));
        assert!(filter.allows(&git("bar-abcdef1")));
    }

    #[test]
    fn remove_entry_test() {
        let mut filter = filter(&["tokio*", "!serde"]);
        filter.remove("!serde");
        assert!(filter.allows(&registry("tokio", "1.0.0")));
        assert!(!filter.allows(&registry("serde", "1.0.0")));
        filter.remove("tokio*");
        assert!(filter.allows(&registry("serde", "1.0.0")));
        assert_eq!(Vec::<String>::from(filter), Vec::<String>::new());
    }
}
