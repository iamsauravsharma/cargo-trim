use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

// Create all list of subcommand options flag using clap
pub(super) fn app() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("trim")
                .author(env!("CARGO_PKG_AUTHORS"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Clean up all .cargo/registry & .cargo/git follow config file data"),
                )
                .arg(
                    Arg::with_name("clear config")
                        .short("c")
                        .long("clear")
                        .help("Clear config data"),
                )
                .arg(
                    Arg::with_name("exclude")
                        .short("e")
                        .long("exclude")
                        .help("Exclude listed crates")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("Crate"),
                )
                .arg(
                    Arg::with_name("exclude-conf")
                        .short("E")
                        .long("exclude-conf")
                        .help("add listed crates to default conf file exclude list")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("Crate"),
                )
                .arg(
                    Arg::with_name("force remove")
                        .short("f")
                        .long("force")
                        .help("Force clear cache without reading conf file"),
                )
                .arg(
                    Arg::with_name("git compress")
                        .short("g")
                        .long("gc")
                        .help("Git compress to reduce size of .cargo")
                        .takes_value(true)
                        .possible_values(&["all", "index", "git", "git-checkout", "git-db"]),
                )
                .arg(
                    Arg::with_name("include")
                        .short("i")
                        .long("include")
                        .help("Include listed crates")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("Crate"),
                )
                .arg(
                    Arg::with_name("include-conf")
                        .short("I")
                        .long("include-conf")
                        .help("add listed crates to default conf file include list")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("Crate"),
                )
                .arg(
                    Arg::with_name("light cleanup")
                        .short("l")
                        .long("light")
                        .help(
                            "Light cleanup repos by removing git checkout and registry source but \
                             stores git db and registry archive for future compilation",
                        ),
                )
                .arg(
                    Arg::with_name("old clean")
                        .short("o")
                        .long("old-clean")
                        .help("Clean old cache crates"),
                )
                .arg(
                    Arg::with_name("orphan clean")
                        .short("x")
                        .long("orphan-clean")
                        .help("Clean orphan cache crates"),
                )
                .arg(
                    Arg::with_name("set directory")
                        .short("s")
                        .multiple(true)
                        .long("set-directory")
                        .value_name("Directory")
                        .help("Set directory of Rust project")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("query size")
                        .short("q")
                        .long("query")
                        .help("Return size of .cargo/cache folders"),
                )
                .arg(
                    Arg::with_name("remove-crate")
                        .short("r")
                        .long("remove")
                        .help("Remove listed crates")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("Crate"),
                )
                .arg(
                    Arg::with_name("top crates")
                        .short("t")
                        .long("top")
                        .help("Show certain number of top crates which have highest size")
                        .takes_value(true)
                        .value_name("number"),
                )
                .arg(
                    Arg::with_name("wipe")
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
                        .value_name("Folder"),
                )
                .subcommand(
                    SubCommand::with_name("config")
                        .about("Query config file data")
                        .alias("c")
                        .arg(
                            Arg::with_name("directory")
                                .short("d")
                                .long("directory")
                                .help("Query about directory data"),
                        )
                        .arg(
                            Arg::with_name("include")
                                .short("i")
                                .long("include")
                                .help("Query about include data"),
                        )
                        .arg(
                            Arg::with_name("exclude")
                                .short("e")
                                .long("exclude")
                                .help("Query about exclude data"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("git")
                        .about("Perform operation only to git related cache file")
                        .alias("g")
                        .arg(
                            Arg::with_name("all")
                                .short("a")
                                .long("all")
                                .help("Clean up all .cargo/git follow config file data"),
                        )
                        .arg(
                            Arg::with_name("force remove")
                                .short("f")
                                .long("force")
                                .help("Force clear cache without reading conf file"),
                        )
                        .arg(
                            Arg::with_name("light cleanup")
                                .short("l")
                                .long("light")
                                .help(
                                    "Light cleanup repos by removing git checkout but stores git \
                                     db for future compilation",
                                ),
                        )
                        .arg(
                            Arg::with_name("old clean")
                                .short("o")
                                .long("old-clean")
                                .help("Clean old cache crates"),
                        )
                        .arg(
                            Arg::with_name("orphan clean")
                                .short("x")
                                .long("orphan-clean")
                                .help("Clean orphan cache crates"),
                        )
                        .arg(
                            Arg::with_name("query size")
                                .short("q")
                                .long("query")
                                .help("Return size of .cargo/git cache folders"),
                        )
                        .arg(
                            Arg::with_name("remove-crate")
                                .short("r")
                                .long("remove")
                                .help("Remove listed crates")
                                .multiple(true)
                                .takes_value(true)
                                .value_name("Crate"),
                        )
                        .arg(
                            Arg::with_name("top crates")
                                .short("t")
                                .long("top")
                                .help(
                                    "Show certain number of top git crates which have highest size",
                                )
                                .takes_value(true)
                                .value_name("number"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("registry")
                        .about("Perform operation only to registry related cache file")
                        .aliases(&["reg", "rg"])
                        .arg(
                            Arg::with_name("all")
                                .short("a")
                                .long("all")
                                .help("Clean up all .cargo/registry follow config file data"),
                        )
                        .arg(
                            Arg::with_name("force remove")
                                .short("f")
                                .long("force")
                                .help("Force clear cache without reading conf file"),
                        )
                        .arg(
                            Arg::with_name("light cleanup")
                                .short("l")
                                .long("light")
                                .help(
                                    "Light cleanup repos by removing registry source but stores \
                                     registry archive for future compilation",
                                ),
                        )
                        .arg(
                            Arg::with_name("old clean")
                                .short("o")
                                .long("old-clean")
                                .help("Clean old cache crates"),
                        )
                        .arg(
                            Arg::with_name("orphan clean")
                                .short("x")
                                .long("orphan-clean")
                                .help("Clean orphan cache crates"),
                        )
                        .arg(
                            Arg::with_name("query size")
                                .short("q")
                                .long("query")
                                .help("Return size of .cargo/registry cache folders"),
                        )
                        .arg(
                            Arg::with_name("remove-crate")
                                .short("r")
                                .long("remove")
                                .help("Remove listed crates")
                                .multiple(true)
                                .takes_value(true)
                                .value_name("Crate"),
                        )
                        .arg(
                            Arg::with_name("top crates")
                                .short("t")
                                .long("top")
                                .help(
                                    "Show certain number of top registry crates which have \
                                     highest size",
                                )
                                .takes_value(true)
                                .value_name("number"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List out crates")
                        .alias("l")
                        .arg(
                            Arg::with_name("all")
                                .short("a")
                                .long("all")
                                .help("list out all installed crate"),
                        )
                        .arg(
                            Arg::with_name("orphan")
                                .short("x")
                                .long("orphan")
                                .help("list out orphan crates"),
                        )
                        .arg(
                            Arg::with_name("old")
                                .short("o")
                                .long("old")
                                .help("List out old crates"),
                        )
                        .arg(
                            Arg::with_name("used")
                                .short("u")
                                .long("use")
                                .help("List out used crates"),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .about("Remove values from config file")
                        .aliases(&["rem", "rm"])
                        .arg(
                            Arg::with_name("directory")
                                .short("d")
                                .long("directory")
                                .help("directory name to be removed")
                                .takes_value(true)
                                .value_name("Folder"),
                        )
                        .arg(
                            Arg::with_name("include")
                                .short("i")
                                .long("include")
                                .help("Remove crate from include")
                                .takes_value(true)
                                .value_name("Crate"),
                        )
                        .arg(
                            Arg::with_name("exclude")
                                .short("e")
                                .long("exclude")
                                .help("Remove crate from exclude")
                                .takes_value(true)
                                .value_name("Crate"),
                        ),
                ),
        )
        .get_matches()
}
