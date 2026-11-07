use anyhow::Result;
use clap::{Args, Parser, ValueEnum};

use self::clean::{Cache, CrateSelection, clean_crates, light_cleanup};
use self::utils::{query_full_width, query_print, source_name_max_width};
use crate::color::{ColorChoice, Paint as _};
use crate::command::registry::clear_empty_index;
use crate::command::target::clean_all_target;
use crate::config_file::ConfigFile;
use crate::dir_path::DirPath;
use crate::installed::Sources;
use crate::list_crate::{CrateList, Selection};
use crate::output::{self, outcome, verbose};
use crate::remove::{RegistryDir, delete_index_cache, remove_path};

mod clean;
mod config;
mod gc;
mod git;
mod list;
mod query;
mod registry;
mod set;
mod target;
mod top;
mod unset;
mod update;
mod utils;

#[derive(Debug, Parser)]
enum SubCommand {
    Config(config::Config),
    Set(set::Set),
    Unset(unset::Unset),
    List(list::List),
    Query(query::Query),
    Top(top::Top),
    Gc(gc::Gc),
    Update(update::Update),
    Git(git::Git),
    Registry(registry::Registry),
    Target(target::Target),
}

/// Output flags shared by every command
#[derive(Debug, Args)]
struct OutputArgs {
    /// Hide progress lines, only results, warnings and errors are printed
    #[arg(
        long,
        short,
        env = "TRIM_QUIET",
        global = true,
        conflicts_with = "verbose"
    )]
    quiet: bool,
    /// Print details to standard error
    ///
    /// Prints used settings, found Cargo.lock files, skipped folders, every
    /// removed path and executed commands
    #[arg(long, short, env = "TRIM_VERBOSE", global = true)]
    verbose: bool,
}

/// Dry run flag of every command which changes something
#[derive(Debug, Args)]
pub(super) struct DryRun {
    /// Show what would be done without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub(super) enabled: bool,
}

impl SubCommand {
    fn run(
        &self,
        dir_path: &DirPath,
        config_file: &mut ConfigFile,
        sources: &Sources,
        crate_list: &CrateList,
        registry_dir: &mut RegistryDir,
    ) -> Result<()> {
        let directory_is_empty = config_file.directory().is_empty();
        match self {
            Self::Config(config) => config.run(config_file, dir_path.config_file())?,
            Self::List(list) => {
                let max_width = source_name_max_width(sources);
                list.run(crate_list, max_width, directory_is_empty);
            }
            Self::Query(query) => query.run(dir_path, crate_list, config_file)?,
            Self::Top(top) => top.run(crate_list, sources, config_file)?,
            Self::Gc(gc) => gc.run(dir_path)?,
            Self::Update(update) => update.run(crate_list.cargo_lock_files().paths())?,
            Self::Set(set) => set.run(config_file)?,
            Self::Unset(unset) => unset.run(config_file)?,
            Self::Git(git) => {
                git.run(dir_path, crate_list, registry_dir, directory_is_empty)?;
            }
            Self::Registry(registry) => {
                registry.run(dir_path, crate_list, registry_dir, directory_is_empty)?;
            }
            Self::Target(target) => target.run(config_file)?,
        }
        Ok(())
    }

    /// whether subcommand is asked for dry run, only subcommands which change
    /// something accept it
    fn dry_run(&self) -> bool {
        match self {
            Self::Config(_) | Self::List(_) | Self::Query(_) | Self::Top(_) => false,
            Self::Set(set) => set.dry_run.enabled,
            Self::Unset(unset) => unset.dry_run.enabled,
            Self::Git(git) => git.dry_run.enabled,
            Self::Registry(registry) => registry.dry_run.enabled,
            Self::Target(target) => target.dry_run.enabled,
            Self::Gc(gc) => gc.dry_run.enabled,
            Self::Update(update) => update.dry_run.enabled,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name= clap::crate_name!(),
    version=clap::crate_version!(),
    propagate_version=true,
    arg_required_else_help=true,
    author=clap::crate_authors!(),
    about=clap::crate_description!()
)]
pub(crate) struct Command {
    /// Clean all crates, empty indexes and target folders
    ///
    /// Removes registry and git crates allowed by filter, then registry indexes
    /// left without crates, then target folders of all projects after
    /// confirmation
    #[arg(long, short)]
    all: bool,
    /// When to color the output
    #[arg(
        long,
        short,
        env = "TRIM_COLOR",
        value_enum,
        default_value_t = ColorChoice::Auto,
        global = true
    )]
    color: ColorChoice,
    /// Add project directory for this run
    #[arg(long, short, env = "TRIM_DIRECTORY", value_name = "path")]
    directory: Option<Vec<String>>,
    #[command(flatten)]
    dry_run: DryRun,
    /// Ignore path while scanning for this run
    ///
    /// A relative path matches trailing path components anywhere, an absolute
    /// path matches the full path and each component may contain `*` (any
    /// characters) and `?` (one character). A folder holding
    /// `.cargo-trim-ignore` is always skipped
    #[arg(long, short, env = "TRIM_IGNORE", value_name = "path")]
    ignore: Option<Vec<String>>,
    /// Add crate filter for this run
    ///
    /// Package spec `[registry:]name[@version]`. Registry and name may contain
    /// `*` (any characters) and `?` (one character), registry matches the
    /// source folder name with or without its hash suffix and version is a
    /// semver requirement (exact for a full version) or a git revision prefix.
    /// Without any plain entry every crate may be cleaned, otherwise
    /// only matching crates are cleaned. An entry starting with `!` is never
    /// cleaned and wins over other entries
    #[arg(long, short, env = "TRIM_FILTER", value_name = "spec")]
    filter: Option<Vec<String>>,
    /// Remove sources but keep archives for offline builds
    ///
    /// Removes registry sources, index caches and git checkouts while registry
    /// archives and git db stay so projects still build without internet
    #[arg(long = "light", short)]
    light_cleanup: bool,
    #[command(flatten)]
    selection: CrateSelection,
    #[command(flatten)]
    output: OutputArgs,
    /// Scan hidden folders for this run
    #[arg(
        long,
        short = 'H',
        env = "TRIM_SCAN_HIDDEN_FOLDER",
        value_name = "bool"
    )]
    scan_hidden_folder: Option<bool>,
    /// Scan target folders for Cargo.lock files for this run
    #[arg(
        long,
        short = 'T',
        env = "TRIM_SCAN_TARGET_FOLDER",
        value_name = "bool"
    )]
    scan_target_folder: Option<bool>,
    /// Days without change after which a project is stale for this run
    ///
    /// Projects without any change for this many days are stale and their
    /// Cargo.lock files no longer mark crates as used, 0 turns it off
    #[arg(long, short, env = "TRIM_STALE_DAYS", value_name = "days")]
    stale_days: Option<u32>,
    /// Delete whole cache folder
    #[arg(long, short, value_enum, value_name = "folder")]
    wipe: Option<Vec<Wipe>>,
    #[command(subcommand)]
    sub: Option<SubCommand>,
}

#[derive(Clone, ValueEnum, Debug)]
enum Wipe {
    Git,
    Checkouts,
    Db,
    Registry,
    Cache,
    Index,
    IndexCache,
    Src,
}

impl Command {
    pub(crate) fn run(&self) -> Result<()> {
        self.color.apply();
        // dry run of main command also covers its subcommand, set once here and
        // read everywhere through `output::is_dry_run`
        output::set(
            self.dry_run.enabled || self.sub.as_ref().is_some_and(SubCommand::dry_run),
            self.output.quiet,
            self.output.verbose,
        );

        let dir_path = DirPath::new()?;
        verbose!("Using config file {}", dir_path.config_file().display());
        let mut config_file = ConfigFile::init(dir_path.config_file())?;
        // apply before building crate list so this run uses the overrides
        self.apply_overrides(&mut config_file)?;
        verbose!("Project directories: {:?}", config_file.directory());

        let sources = Sources::new(dir_path.index_dir(), dir_path.db_dir())?;
        let crate_list = CrateList::create_list(&dir_path, &config_file, &sources)?;
        verbose!(
            "Found {} Cargo.lock file(s), {} registry and {} git crate(s) installed",
            crate_list.cargo_lock_files().paths().len(),
            crate_list.registry(Selection::Installed).len(),
            crate_list.git(Selection::Installed).len(),
        );

        self.run_maintenance(&dir_path);

        // read registry index caches only after maintenance which may remove
        // them
        let mut registry_dir = RegistryDir::new(
            dir_path.index_dir(),
            &crate_list.registry(Selection::Installed),
        )?;
        self.run_clean(&dir_path, &config_file, &crate_list, &mut registry_dir)?;

        if let Some(sub_command) = &self.sub {
            sub_command.run(
                &dir_path,
                &mut config_file,
                &sources,
                &crate_list,
                &mut registry_dir,
            )?;
        }
        Ok(())
    }

    /// apply config values given for this run only
    fn apply_overrides(&self, config_file: &mut ConfigFile) -> Result<()> {
        for directory in self.directory.iter().flatten() {
            config_file.add_directory(directory, false)?;
        }
        for ignore in self.ignore.iter().flatten() {
            config_file.add_ignore(ignore.trim_end_matches(std::path::MAIN_SEPARATOR), false)?;
        }
        for filter in self.filter.iter().flatten() {
            config_file.add_filter(filter, false)?;
        }
        if let Some(scan_hidden_folder) = self.scan_hidden_folder {
            config_file.set_scan_hidden_folder(scan_hidden_folder, false)?;
        }
        if let Some(scan_target_folder) = self.scan_target_folder {
            config_file.set_scan_target_folder(scan_target_folder, false)?;
        }
        if let Some(stale_days) = self.stale_days {
            config_file.set_stale_days(stale_days, false)?;
        }
        Ok(())
    }

    /// light cleanup and wipe
    fn run_maintenance(&self, dir_path: &DirPath) {
        if self.light_cleanup {
            light_cleanup(Cache::Every, dir_path);
        }
        for wipe in self.wipe.iter().flatten() {
            wipe_directory(wipe, dir_path);
        }
    }

    /// clean chosen crates, and with `--all` also empty indexes and target
    /// folders
    fn run_clean(
        &self,
        dir_path: &DirPath,
        config_file: &ConfigFile,
        crate_list: &CrateList,
        registry_dir: &mut RegistryDir,
    ) -> Result<()> {
        let directory_is_empty = config_file.directory().is_empty();
        self.selection
            .clean(Cache::Every, registry_dir, crate_list, directory_is_empty)?;
        if self.all {
            clean_crates(Cache::Every, "", registry_dir, crate_list, Selection::All)?;
            clear_empty_index(dir_path, crate_list)?;
            clean_all_target(config_file)?;
        }
        Ok(())
    }
}

// wipe certain directory
fn wipe_directory(wipe: &Wipe, dir_path: &DirPath) {
    // a wiped folder is printed once below, its path only with verbose output
    let path = match wipe {
        Wipe::Git => Some(dir_path.git_dir()),
        Wipe::Checkouts => Some(dir_path.checkout_dir()),
        Wipe::Db => Some(dir_path.db_dir()),
        Wipe::Registry => Some(dir_path.registry_dir()),
        Wipe::Cache => Some(dir_path.cache_dir()),
        Wipe::Index => Some(dir_path.index_dir()),
        Wipe::Src => Some(dir_path.src_dir()),
        // index caches are spread over every registry
        Wipe::IndexCache => None,
    };
    let has_failed = match path {
        Some(path) => remove_path(path).is_err(),
        None => delete_index_cache(dir_path.index_dir()).is_err(),
    };
    if has_failed {
        println!("Failed to remove {wipe:?} directory");
    } else {
        outcome!("{} {wipe:?} directory", "Removed".red());
    }
}
