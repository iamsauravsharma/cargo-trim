# CARGO-TRIM

**Project status & info:**

|                    License                     |              Crates Version               |
| :--------------------------------------------: | :---------------------------------------: |
| [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] |

Binary application for cleaning **\$CARGO_HOME/registry** & **\$CARGO_HOME/git** folder support orphan and old crates cleaning
Default \$CARGO_HOME is set as \$HOME/.cargo

### Install

For installation of cargo-trim:-
You can install from crates.io registry

```
cargo install cargo-trim
```

**OR**

For bleeding edge latest development version:

```
cargo install --git https://github.com/iamsauravsharma/cargo-trim
```

### Setup

Initially you need to set up cargo-trim for some commands such as orphan clean to work properly.
To set up cargo-trim to properly work for all projects related to rust lang go to directory where all projects lies and run
`cargo trim init`. You can even run this command multiple time to add multiple directory.
Or run command `cargo trim set -d <directory-path>`. All projects inside the directory will be used for listing orphan crates. So
if there are any projects in the other directory then their dependencies will be classified as orphan crates if not used by any
other crates present inside added directory.

### Command output

All cargo-trim can be run using `cargo trim <command>`
`cargo trim --help` will produce out following output which list out all of available command for cargo-trim

```
Binary application to cleanup $CARGO_HOME cache

Usage: cargo-trim [OPTIONS] [COMMAND]

Commands:
  init      Initialize current working directory as cargo trim directory
  clear     Clear current working directory from cargo cache config
  config    Query about config file data used by CLI
  set       Set config file values
  unset     Unset values from config file
  list      List out crates
  git       Perform operation only to git related cache file
  registry  Perform operation only to registry related cache file
  help      Print this message or the help of the given subcommand(s)

Options:
  -a, --all                    Clean up all registry & git crates
  -d, --directory <DIRECTORY>  Extra list of directory of Rust projects for current command [env: TRIM_DIRECTORY=]
  -n, --dry-run                Run command in dry run mode to see what would be done
  -g, --gc <GIT_COMPRESS>      Git compress to reduce size of .cargo (git command required) [possible values: aggressive-checkout, aggressive-db, aggressive-index, checkout, db, index]
  -i, --ignore <IGNORE>        Extra list of ignore file name which should be ignored for current command [env: TRIM_IGNORE=]
  -l, --light                  Light cleanup without removing files required for future compilation without internet
      --no-scan-hidden-folder  Do not scan hidden folder for current command. Takes precedence over scan-hidden-folder [env: TRIM_NOT_SCAN_HIDDEN_FOLDER=]
      --no-scan-target-folder  Do not scan target folder for current command. Takes precedence over scan-target-folder [env: TRIM_NOT_SCAN_TARGET_FOLDER=]
  -o, --old                    Clean old cache crates
  -z, --old-orphan             Clean crates which is both old and orphan
  -x, --orphan                 Clean orphan cache crates i.e all crates which are not present in lock file generated till now
  -q, --query                  Return size of different .cargo/cache folders
      --scan-hidden-folder     Scan hidden folder for current command [env: TRIM_SCAN_HIDDEN_FOLDER=]
      --scan-target-folder     Scan target folder for current command [env: TRIM_SCAN_TARGET_FOLDER=]
  -t, --top <TOP>              Show certain number of top crates which have highest size
  -u, --update                 Generate and Update Cargo.lock file present inside config directory folder path
  -w, --wipe <WIPE>            Wipe folder [possible values: git, checkouts, db, registry, cache, index, index-cache, src]
  -h, --help                   Print help
  -V, --version                Print version
```

### Configuration

cargo-trim store its config file to config directory of OS and name config file as `cargo_trim_config.toml`.
In file cargo-trim stores different information for scanning projects as well as listing crates.
To list where cargo trim is storing config file you can use `cargo trim config --location`
Below are a list of configuration and their default value a well as corresponding env variable that can be used for setting
same value without editing config file

#### 1. directory

**default: []**

**env: TRIM_DIRECTORY**

List of directory to scan for a Rust projects

#### 2. **ignore_file_name**

**default: []**

**env: TRIM_IGNORE**

File/Directory name which should be ignored while scanning through directories. This can be used to exclude some folder
such as npm node_modules folder or python venv folder if the directory present in config contains such folder it will help
to significantly increase performance.

#### 3. **scan_hidden_folder**

**default: false**

**env: TRIM_SCAN_HIDDEN_FOLDER**

Whether to scan hidden folder(i.e. folder which start with dot notation) or not

#### 4. **scan_target_folder**

**default: false**

**env: TRIM_SCAN_TARGET_FOLDER**

Whether to scan target folder. Currently, it searches for environment variable `CARGO_BUILD_TARGET_DIR` or `CARGO_TARGET_DIR`
or set default value as target to determine a target folder name

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg?style=for-the-badge
[license_link]: LICENSE
[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg?style=for-the-badge
[cratesio_link]: https://crates.io/crates/cargo-trim
