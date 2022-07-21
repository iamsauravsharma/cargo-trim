# CARGO-TRIM

**Project status & info:**

| License | Crates Version |
| :-----: | :------------: |
| [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] |

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

### Setup
Initially you need to set up cargo-trim for some commands such as orphan clean to work properly.
To set up cargo-trim to properly work for all projects related to rust lang go to directory where all projects lies and run
`cargo trim init`. You can even run this command multiple time to add multiple directory.
Or run command `cargo trim -d <directory-path>`. All projects inside the directory will be used for listing orphan crates. So
if there are any projects in the other directory then their dependencies will be classified as orphan crates if not used by any
other crates present inside added directory.

### Command output
All cargo-trim can be run using `cargo trim <command>`
`cargo trim --help` will produce out following output which list out all of available command for cargo-trim
```
cargo-trim 0.8.1
Saurav Sharma <appdroiddeveloper@gmail.com>
Binary application to cleanup $CARGO_HOME cache

USAGE:
    cargo-trim [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -a, --all
            Clean up all registry & git crates

    -d, --directory <DIRECTORY>
            Extra list of directory of Rust projects for current command [env: TRIM_DIRECTORY=]

    -g, --gc <GIT_COMPRESS>
            Git compress to reduce size of .cargo (git command required) [possible values: all,
            index, git, git-checkout, git-db]

    -h, --help
            Print help information

    -i, --ignore <IGNORE>
            Extra list of ignore file name which should be ignored for current command [env:
            TRIM_IGNORE=]

    -l, --light
            Light cleanup repo by removing git checkout and registry source but stores git db and
            registry archive for future compilation without internet requirement

    -n, --dry-run
            Run command in dry run mode to see what would be done

    -o, --old
            Clean old cache crates

    -q, --query
            Return size of different .cargo/cache folders

    -r, --remove <crate>
            Remove provided crates from registry or git

        --scan-hidden-folder <SCAN_HIDDEN_FOLDER>
            Whether to scan hidden folder for current command [env: TRIM_SCAN_HIDDEN_FOLDER=]
            [possible values: true, false]

        --scan-target-folder <SCAN_TARGET_FOLDER>
            Whether to scan target folder for current command [env: TRIM_SCAN_TARGET_FOLDER=]
            [possible values: true, false]

    -t, --top <number>
            Show certain number of top crates which have highest size

    -u, --update
            Generate and Update Cargo.lock file present inside config directory folder path

    -V, --version
            Print version information

    -w, --wipe <folder>
            Wipe folder [possible values: git, checkouts, db, registry, cache, index, index-cache,
            src]

    -x, --orphan
            Clean orphan cache crates i.e all crates which are not present in lock file generated
            till now use cargo trim -u to guarantee your all project generate lock file

    -z, --old-orphan
            Clean crates which is both old and orphan

SUBCOMMANDS:
    clear       Clear current working directory from cargo cache config
    config      Query about config file data used by CLI
    git         Perform operation only to git related cache file
    help        Print this message or the help of the given subcommand(s)
    init        Initialize current working directory as cargo trim directory
    list        List out crates
    registry    Perform operation only to registry related cache file
    set         Set config file values
    unset       Unset values from config file
```

### Configuration
cargo-trim store its config file to config directory of OS and name config file as `cargo_trim_config.toml`.
In file cargo-trim stores different information for scanning projects as well as listing crates.
To list where cargo trim is storing config file you can use `cargo trim config --location`
Below are a list of configuration and their default value a well as corresponding env variable that can be used for setting
same value without editing config file

#### 1. directory

__default: []__

__env: TRIM_DIRECTORY__

List of directory to scan for a Rust projects

#### 2. __ignore_file_name__

__default: []__

__env: TRIM_IGNORE__

File/Directory name which should be ignored while scanning through directories. This can be used to exclude some folder
such as npm node_modules folder or python venv folder if the directory present in config contains such folder it will help
to significantly increase performance.

#### 3. __scan_hidden_folder__

__default: false__

__env: TRIM_SCAN_HIDDEN_FOLDER__

Whether to scan hidden folder(i.e. folder which start with dot notation) or not

#### 4. __scan_target_folder__

__default: false__

__env: TRIM_SCAN_TARGET_FOLDER__

Whether to scan target folder. Currently, it searches for environment variable `CARGO_BUILD_TARGET_DIR` or `CARGO_TARGET_DIR`
or set default value as target to determine a target folder name

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg?style=for-the-badge
[license_link]: LICENSE

[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg?style=for-the-badge
[cratesio_link]: https://crates.io/crates/cargo-trim
