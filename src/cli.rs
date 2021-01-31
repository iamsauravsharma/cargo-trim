use clap::{App, AppSettings, Arg, ArgGroup, SubCommand};

// Create all list of subcommand options flag using clap
#[allow(clippy::too_many_lines)]
pub(super) fn init() -> App<'static, 'static> {
    let all = Arg::with_name("all").short("a").long("all");
    let all_trim = all.clone().help("Clean up all registry & git crates");
    let all_git = all.clone().help("Clean up all git crates");
    let all_registry = all.clone().help("Clean up all registry crates");
    let all_list = all.help("list out all installed crate");

    let directory = Arg::with_name("directory").short("d").long("directory");
    let directory_config = directory.clone().help("Query about directory data");
    let directory_remove = directory
        .clone()
        .help("Directory to be removed")
        .takes_value(true)
        .multiple(true)
        .value_name("directory");
    let directory_trim = directory
        .value_name("directory")
        .help(
            "Set directory of Rust project [use TRIM_DIRECTORY environment variable for creating \
             directory list without editing configuration file]",
        )
        .multiple(true)
        .takes_value(true);

    let dry_run = Arg::with_name("dry run")
        .short("n")
        .long("dry-run")
        .help("Run command in dry run mode to see what would be removed");

    let git_compress = Arg::with_name("git compress")
        .short("g")
        .long("gc")
        .help("Git compress to reduce size of .cargo")
        .takes_value(true)
        .possible_values(&["all", "index", "git", "git-checkout", "git-db"]);

    let ignore_file_name = Arg::with_name("ignore_file_name").short("i").long("ignore");
    let ignore_file_name_config = ignore_file_name
        .clone()
        .help("Query about ignored file name data");
    let ignore_file_name_remove = ignore_file_name
        .clone()
        .help("Remove file name from ignore file name list")
        .takes_value(true)
        .multiple(true)
        .value_name("file");
    let ignore_file_name_trim = ignore_file_name
        .takes_value(true)
        .multiple(true)
        .value_name("file")
        .help(
            "Add file name/directory name to ignore list in configuration file which are ignored \
             while scanning Cargo.toml file [use TRIM_IGNORE_FILE_NAME environment variable for \
             creating ignore file name list without editing configuration file]",
        );

    let light_cleanup = Arg::with_name("light cleanup").short("l").long("light");
    let light_cleanup_trim = light_cleanup.clone().help(
        "Light cleanup repo by removing git checkout and registry source but stores git db and \
         registry archive for future compilation without internet requirement",
    );

    let light_cleanup_git = light_cleanup.clone().help(
        "Light cleanup repo by removing git checkout but stores git db for future compilation",
    );
    let light_cleanup_registry = light_cleanup.help(
        "Light cleanup repo by removing registry source but stores registry archive for future \
         compilation",
    );

    let location = Arg::with_name("location")
        .short("l")
        .long("location")
        .help("Return config file location");

    let old = Arg::with_name("old")
        .short("o")
        .long("old")
        .help("List out old crates");
    let old_orphan = Arg::with_name("old-orphan")
        .short("z")
        .long("old-orphan")
        .help("List out crates which is both old and orphan");

    let old_clean = Arg::with_name("old clean")
        .short("o")
        .long("old-clean")
        .help("Clean old cache crates");
    let old_orphan_clean = Arg::with_name("old-orphan-clean")
        .short("z")
        .long("old-orphan-clean")
        .help("Clean crates which is both old and orphan");

    let orphan = Arg::with_name("orphan")
        .short("x")
        .long("orphan")
        .help("List out orphan crates");

    let orphan_clean = Arg::with_name("orphan clean")
        .short("x")
        .long("orphan-clean")
        .help(
            "Clean orphan cache crates i.e all crates which are not present in lock file \
             generated till now use cargo trim -u to guarantee your all project generate lock file",
        );

    let print_config = Arg::with_name("print config")
        .short("p")
        .long("print")
        .help("Print/Display config file content");

    let query_size = Arg::with_name("query size").short("q").long("query");
    let query_size_trim = query_size
        .clone()
        .help("Return size of different .cargo/cache folders");
    let query_size_git = query_size
        .clone()
        .help("Return size of different .cargo/git cache folders");
    let query_size_registry =
        query_size.help("Return size of different .cargo/registry cache folders");

    let remove_crate = Arg::with_name("remove-crate")
        .short("r")
        .long("remove")
        .takes_value(true)
        .multiple(true)
        .value_name("crate");
    let remove_crate_trim = remove_crate
        .clone()
        .help("Remove provided crates from registry or git");
    let remove_crate_registry = remove_crate
        .clone()
        .help("Remove provided crates from registry");
    let remove_crate_git = remove_crate.help("Remove provided crates from git");

    let top_crate = Arg::with_name("top crates")
        .short("t")
        .long("top")
        .takes_value(true)
        .value_name("number");
    let top_crate_trim = top_crate
        .clone()
        .help("Show certain number of top crates which have highest size");
    let top_crate_git = top_crate
        .clone()
        .help("Show certain number of top git crates which have highest size");
    let top_crates_registry =
        top_crate.help("Show certain number of top registry crates which have highest size");

    let update = Arg::with_name("update")
        .short("u")
        .long("update")
        .help("Generate and Update Cargo.lock file present inside config directory folder path");

    let used = Arg::with_name("used")
        .short("u")
        .long("use")
        .help("List out used crates");

    let wipe = Arg::with_name("wipe")
        .short("w")
        .long("wipe")
        .help("Wipe folder")
        .possible_values(&[
            "git",
            "checkouts",
            "db",
            "registry",
            "cache",
            "index",
            "index-cache",
            "src",
        ])
        .takes_value(true)
        .multiple(true)
        .value_name("folder");

    App::new(env!("CARGO_PKG_NAME"))
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .settings(&[
            AppSettings::GlobalVersion,
            AppSettings::SubcommandRequired,
            AppSettings::ArgRequiredElseHelp,
        ])
        .subcommand(
            SubCommand::with_name("trim")
                .author(env!("CARGO_PKG_AUTHORS"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .args(&[
                    all_trim,
                    dry_run.clone(),
                    git_compress,
                    light_cleanup_trim,
                    old_clean.clone(),
                    old_orphan_clean.clone(),
                    orphan_clean.clone(),
                    query_size_trim,
                    remove_crate_trim,
                    directory_trim,
                    ignore_file_name_trim,
                    top_crate_trim,
                    update,
                    wipe,
                ])
                .subcommand(
                    SubCommand::with_name("init")
                        .about("Initialize current working directory as cargo trim directory"),
                )
                .subcommand(
                    SubCommand::with_name("clear")
                        .about("Clear current working directory from cargo cache config")
                        .arg(dry_run.clone()),
                )
                .subcommand(
                    SubCommand::with_name("config")
                        .about("Query about config file data used by CLI")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .args(&[
                            directory_config,
                            ignore_file_name_config,
                            location,
                            print_config,
                        ]),
                )
                .subcommand(
                    SubCommand::with_name("git")
                        .about("Perform operation only to git related cache file")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .args(&[
                            all_git,
                            dry_run.clone(),
                            light_cleanup_git,
                            old_clean.clone(),
                            old_orphan_clean.clone(),
                            orphan_clean.clone(),
                            query_size_git,
                            remove_crate_git,
                            top_crate_git,
                        ])
                        .group(ArgGroup::with_name("crate detail required").args(&[
                            "all",
                            "query size",
                            "old clean",
                            "old-orphan-clean",
                            "orphan clean",
                            "remove-crate",
                            "top crates",
                        ])),
                )
                .subcommand(
                    SubCommand::with_name("registry")
                        .about("Perform operation only to registry related cache file")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .args(&[
                            all_registry,
                            dry_run.clone(),
                            light_cleanup_registry,
                            old_clean,
                            old_orphan_clean,
                            orphan_clean,
                            query_size_registry,
                            remove_crate_registry,
                            top_crates_registry,
                        ])
                        .group(ArgGroup::with_name("crate detail required").args(&[
                            "all",
                            "query size",
                            "old clean",
                            "old-orphan-clean",
                            "orphan clean",
                            "remove-crate",
                            "top crates",
                        ])),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List out crates")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .args(&[all_list, old, old_orphan, orphan, used]),
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .about("Remove values from config file")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .args(&[directory_remove, dry_run, ignore_file_name_remove]),
                )
                .groups(&[
                    ArgGroup::with_name("config file modifier")
                        .args(&["directory", "ignore_file_name"]),
                    ArgGroup::with_name("crate detail required").args(&[
                        "all",
                        "query size",
                        "old clean",
                        "old-orphan-clean",
                        "orphan clean",
                        "remove-crate",
                        "top crates",
                        "update",
                    ]),
                ]),
        )
}
