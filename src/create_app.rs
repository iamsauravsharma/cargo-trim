use clap::{App, Arg, ArgMatches, SubCommand};

// Create all list of subcommand options flag using clap
pub(super) fn app() -> ArgMatches<'static> {
    App::new("cargo trim")
        .version("0.1.0")
        .author("Saurav Sharma <appdroiddeveloper@gmail.com>")
        .about("Clean cache from .cargo/registry")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Clean up all .cargo/registry follow config file data"),
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
                .short("d")
                .multiple(true)
                .long("directory")
                .value_name("Directory")
                .help("Set directory of Rust project")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("query size")
                .short("q")
                .long("size")
                .help("Return size of .cargo/registry"),
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
            Arg::with_name("wipe")
                .short("w")
                .long("wipe")
                .help("Wipe folder expected value : registry, cache, index, src")
                .takes_value(true)
                .value_name("Folder"),
        )
        .subcommand(
            SubCommand::with_name("query")
                .about("Query config file data")
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
            SubCommand::with_name("list")
                .about("List out all of installed crates")
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
        )
        .get_matches()
}
