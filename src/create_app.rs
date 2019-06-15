use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

// Create all list of subcommand options flag using clap
pub(super) fn app() -> ArgMatches<'static> {
    let all = Arg::with_name("all").short("a").long("all");
    let all_trim = all
        .clone()
        .help("Clean up all .cargo/registry & .cargo/git follow config file data");
    let all_git = all
        .clone()
        .help("Clean up all .cargo/git follow config file data");
    let all_registry = all
        .clone()
        .help("Clean up all .cargo/registry follow config file data");
    let all_list = all.clone().help("list out all installed crate");

    let clear_config = Arg::with_name("clear config")
        .short("c")
        .long("clear")
        .help("Clear config data");

    let directory = Arg::with_name("directory").short("d").long("directory");
    let directory_config = directory.clone().help("Query about directory data");
    let directory_remove = directory
        .clone()
        .help("directory name to be removed")
        .takes_value(true)
        .value_name("Folder");

    let exclude = Arg::with_name("exclude").short("e").long("exclude");
    let exclude_config = exclude.clone().help("Query about exclude data");
    let exclude_remove = exclude
        .clone()
        .help("Remove crate from exclude")
        .takes_value(true)
        .value_name("Crate");

    let exclude_conf = exclude
        .clone()
        .help(
            "add listed crates to default conf file exclude list [use TRIM_EXCLUDE environment \
             variable for creating exclude list without editing conf file]",
        )
        .multiple(true)
        .takes_value(true)
        .value_name("Crate");

    let force_remove = Arg::with_name("force remove")
        .short("f")
        .long("force")
        .help("Force clear cache without reading conf file");

    let git_compress = Arg::with_name("git compress")
        .short("g")
        .long("gc")
        .help("Git compress to reduce size of .cargo")
        .takes_value(true)
        .possible_values(&["all", "index", "git", "git-checkout", "git-db"]);

    let include = Arg::with_name("include").short("i").long("include");
    let include_config = include.clone().help("Query about include data");
    let include_remove = include
        .clone()
        .help("Remove crate from include")
        .takes_value(true)
        .value_name("Crate");

    let include_conf = include
        .clone()
        .help(
            "add listed crates to default conf file include list [use TRIM_INCLUDE environment \
             variable for creating include list without editing conf file]",
        )
        .multiple(true)
        .takes_value(true)
        .value_name("Crate");

    let light_cleanup = Arg::with_name("light cleanup").short("l").long("light");
    let light_cleanup_trim = light_cleanup.clone().help(
        "Light cleanup repos by removing git checkout and registry source but stores git db and \
         registry archive for future compilation",
    );

    let light_cleanup_git = light_cleanup.clone().help(
        "Light cleanup repos by removing git checkout but stores git db for future compilation",
    );
    let light_cleanup_registry = light_cleanup.clone().help(
        "Light cleanup repos by removing registry source but stores registry archive for future \
         compilation",
    );

    let old = Arg::with_name("old")
        .short("o")
        .long("old")
        .help("list out old crates");

    let old_clean = Arg::with_name("old clean")
        .short("o")
        .long("old-clean")
        .help("Clean old cache crates");

    let orphan = Arg::with_name("orphan")
        .short("x")
        .long("orphan")
        .help("list out orphan crates");

    let orphan_clean = Arg::with_name("orphan clean")
        .short("x")
        .long("orphan-clean")
        .help("Clean orphan cache crates");

    let query_size = Arg::with_name("query size").short("q").long("query");
    let query_size_trim = query_size
        .clone()
        .help("Return size of .cargo/cache folders");
    let query_size_git = query_size
        .clone()
        .help("Return size of .cargo/git cache folders");
    let query_size_registry = query_size
        .clone()
        .help("Return size of .cargo/registry cache folders");

    let remove_crate = Arg::with_name("remove-crate")
        .short("r")
        .long("remove")
        .help("Remove listed crates")
        .multiple(true)
        .takes_value(true)
        .value_name("Crate");

    let set_directory = Arg::with_name("set directory")
        .short("s")
        .multiple(true)
        .long("set-directory")
        .value_name("Directory")
        .help(
            "Set directory of Rust project [use TRIM_DIRECTORY environment variable for creating \
             directory list without editing conf file]",
        )
        .takes_value(true);

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
    let top_crates_registry = top_crate
        .clone()
        .help("Show certain number of top registry crates which have highest size");

    let update = Arg::with_name("update")
        .short("u")
        .long("update")
        .help("Update Cargo.lock file present inside config directory folder path");

    let used = Arg::with_name("used")
        .short("u")
        .long("use")
        .help("list out used crates");

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
            "src",
        ])
        .takes_value(true)
        .value_name("Folder");

    App::new(env!("CARGO_PKG_NAME"))
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("trim")
                .author(env!("CARGO_PKG_AUTHORS"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .args(&[
                    all_trim,
                    clear_config,
                    exclude_conf,
                    force_remove.clone(),
                    git_compress,
                    include_conf,
                    light_cleanup_trim,
                    old_clean.clone(),
                    orphan_clean.clone(),
                    query_size_trim,
                    remove_crate.clone(),
                    set_directory,
                    top_crate_trim,
                    update,
                    wipe,
                ])
                .subcommand(
                    SubCommand::with_name("init")
                        .about(
                            "Initialize current working directory as cargo trim directory [alias: \
                             \"in\"]",
                        )
                        .alias("in"),
                )
                .subcommand(
                    SubCommand::with_name("clear")
                        .about(
                            "Clear current working directory from cargo cache config [alias: \
                             \"clr\"]",
                        )
                        .alias("clr"),
                )
                .subcommand(
                    SubCommand::with_name("config")
                        .about("Query config file data [alias: \"c\"]")
                        .alias("c")
                        .args(&[directory_config, exclude_config, include_config]),
                )
                .subcommand(
                    SubCommand::with_name("git")
                        .about("Perform operation only to git related cache file [alias: \"g\"]")
                        .alias("g")
                        .args(&[
                            all_git,
                            force_remove.clone(),
                            light_cleanup_git,
                            old_clean.clone(),
                            orphan_clean.clone(),
                            query_size_git,
                            remove_crate.clone(),
                            top_crate_git,
                        ]),
                )
                .subcommand(
                    SubCommand::with_name("registry")
                        .about(
                            "Perform operation only to registry related cache file [alias: \
                             \"reg\"]",
                        )
                        .alias("reg")
                        .args(&[
                            all_registry,
                            force_remove.clone(),
                            light_cleanup_registry,
                            old_clean.clone(),
                            orphan_clean.clone(),
                            query_size_registry,
                            remove_crate.clone(),
                            top_crates_registry,
                        ]),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List out crates [alias: \"l\"]")
                        .alias("l")
                        .args(&[all_list, old, orphan, used]),
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .about("Remove values from config file [alias: \"rm\"]")
                        .alias("rm")
                        .args(&[directory_remove, exclude_remove, include_remove]),
                ),
        )
        .get_matches()
}
