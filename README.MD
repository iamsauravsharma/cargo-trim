# CARGO-TRIM

**Project status & info:**

| Travis Build Status | License | Crates Version |
| :-----------------: | :-----: | :------------: |
| [![Travis Build Status][build_badge]][build_link] | [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] |

Binary application for cleaning __\$CARGO_HOME/registry__  & __\$CARGO_HOME/git__ folder support orphan and old crates cleaning
Default \$CARGO_HOME is set as \$HOME/.cargo

### Install

For installation of cargo-trim:-
You can install from crates.io registry
```
    cargo install cargo-trim
```
__OR__

For bleeding edge latest development version:
```
    cargo install --git https://github.com/iamsauravsharma/cargo-trim
```

By default cargo-trim print out colored output if you want to use a non colored output you can enable out no-color feature
```
    cargo install cargo-trim --features no-color
```

### Setup
Initially you need to setup cargo-trim for some of commands such as orphan clean to work properly.
To setup cargo-trim to properly work for all projects related to rust lang go to directory where all projects lies and run
`cargo trim init`. You can even run this command multiple time to add multiple directory.
or simple run command `cargo trim -d <directory-path>`. All projects inside directory will be used for listing orphan crates. So
if there is any projects in other directory then their dependencies will be classified as orphan crates. If not used by any
other crates.

### Command output
All cargo-trim can be run using `cargo trim <command>`
`cargo trim --help` will produce out following output which list out all of available command for cargo-trim
```
cargo-trim 0.6.1
Saurav Sharma <appdroiddeveloper@gmail.com>
Binary application to cleanup $CARGO_HOME cache

USAGE:
    cargo trim [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -a, --all                 Clean up all registry & git crates
    -n, --dry-run             Run command in dry run mode to see what would be removed
    -h, --help                Prints help information
    -l, --light               Light cleanup repo by removing git checkout and registry source but stores git db and
                              registry archive for future compilation without internet requirement
    -o, --old-clean           Clean old cache crates
    -z, --old-orphan-clean    Clean crates which is both old and orphan
    -x, --orphan-clean        Clean orphan cache crates i.e all crates which are not present in lock file generated till
                              now use cargo trim -u to guarantee your all project generate lock file
    -q, --query               Return size of different .cargo/cache folders
    -u, --update              Generate and Update Cargo.lock file present inside config directory folder path
    -V, --version             Prints version information

OPTIONS:
    -d, --directory <directory>...    Set directory of Rust project [use TRIM_DIRECTORY environment variable for
                                      creating directory list without editing configuration file]
    -g, --gc <git compress>           Git compress to reduce size of .cargo [possible values: all, index, git, git-
                                      checkout, git-db]
    -i, --ignore <file>...            Add file name/directory name to ignore list in configuration file which are
                                      ignored while scanning Cargo.toml file [use TRIM_IGNORE_FILE_NAME environment
                                      variable for creating ignore file name list without editing configuration file]
    -r, --remove <crate>...           Remove provided crates from registry or git
    -t, --top <number>                Show certain number of top crates which have highest size
    -w, --wipe <folder>...            Wipe folder [possible values: git, checkouts, db, registry, cache, index, index-
                                      cache, src]

SUBCOMMANDS:
    clear       Clear current working directory from cargo cache config
    config      Query about config file data used by CLI
    git         Perform operation only to git related cache file
    help        Prints this message or the help of the given subcommand(s)
    init        Initialize current working directory as cargo trim directory
    list        List out crates
    registry    Perform operation only to registry related cache file
    remove      Remove values from config file
```

### Configuration
cargo-trim store its config file to config directory of OS and name config file as `cargo_trim_config.toml`.
In file cargo-trim stores different information for scanning projects as well as listing crates.
To list where cargo trim is storing config file you can use `cargo trim config --location`
Below are list of configuration and their default value a well as corresponding env variable that can be used for setting
same value without editing config file

##### 1. directory (default: [], env: TRIM_DIRECTORY)
List of directory to scan for a Rust projects

##### 2. __ignore_file_name__ (default: [], env: TRIM_IGNORE_FILE_NAME)
File/Directory name which should be ignored while scanning through directories. This can be used to exclude some folder
such as npm node_modules folder or python venv folder if directory present in config contains such folder. It will help
to significantly increase performance.

##### 3. __scan_hidden_folder__ (default: false. env: TRIM_SCAN_HIDDEN_FOLDER)
Whether to scan hidden folder(i.e folder which start with dot notation) or not

##### 4. __scan_target_folder__ (default: false, env: TRIM_SCAN_TARGET_FOLDER)
Whether to scan target folder. Currently it search for environment variable `CARGO_BUILD_TARGET_DIR` or `CARGO_TARGET_DIR`
or set default value as target to determine a target folder name

[git_link]: https://github.com/iamsauravsharma/cargo-trim

[build_badge]: https://img.shields.io/travis/com/iamsauravsharma/cargo-trim.svg?logo=travis
[build_link]: https://travis-ci.com/iamsauravsharma/cargo-trim

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg
[license_link]: LICENSE

[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg
[cratesio_link]: https://crates.io/crates/cargo-trim
