# CARGO-TRIM

**Project status & info:**

|                    License                     |              Crates Version               |
| :--------------------------------------------: | :---------------------------------------: |
| [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] |

`cargo-trim` is a binary application designed to clean up the **\$CARGO_HOME/registry** & **\$CARGO_HOME/git** folders,
supporting orphaned and outdated crate removal.
The default value for $CARGO_HOME is $HOME/.cargo.

### Install

You can install `cargo-trim` directly from the crates.io registry:

```
cargo install cargo-trim
```

**OR**

For bleeding edge latest development version:

```
cargo install --git https://github.com/iamsauravsharma/cargo-trim
```

### Setup

To set up cargo-trim, initialize it in the directory where your Rust projects reside using:

```bash
cargo trim init
```

This command can be run multiple times to add multiple directories.

Alternatively, you can specify a directory manually with:

```bash
cargo trim set -d <directory-path>
```

This setup is crucial for enabling commands like orphan cleaning, as it allows `cargo-trim` to track and manage crates across your projects.

### Commands
You can run `cargo-trim` commands using:
```bash
cargo trim <command>
```
For a full list of available commands, use:

```bash
cargo trim --help
```

This will display the following output:
```
Binary application to cleanup $CARGO_HOME cache

Usage: cargo-trim [OPTIONS] [COMMAND]

Commands:
  init      Initialize current working directory as cargo trim directory
  clear     Clear current working directory from cargo cache config
  config    Query about config file data used by CLI
  set       Set config file values
  unset     Unset values from config file
  list      List crates
  git       Perform operation only to git related cache file
  registry  Perform operation only to registry related cache file
  help      Print this message or the help of the given subcommand(s)

Options:
  -a, --all                    Clean up all registry & git crates
      --clear-empty-index      Clear all empty index directory
  -d, --directory <DIRECTORY>  Extra list of directory of Rust projects for current command [env: TRIM_DIRECTORY=]
  -n, --dry-run                Run command in dry run mode to see what would be done
  -g, --gc <GIT_COMPRESS>      Git compress to reduce size of .cargo (git command required) [possible values: aggressive-checkout, aggressive-db, aggressive-index, checkout, db, index]
  -i, --ignore <IGNORE>        Extra list of ignore file name which should be ignored for current command [env: TRIM_IGNORE=]
  -l, --light                  Light cleanup without removing files required for future compilation without internet
      --no-scan-hidden-folder  Do not scan hidden folder for current command. Takes precedence over scan-hidden-folder [env: TRIM_NOT_SCAN_HIDDEN_FOLDER=]
      --no-scan-target-folder  Do not scan target folder for current command. Takes precedence over scan-target-folder [env: TRIM_NOT_SCAN_TARGET_FOLDER=]
  -o, --old                    Clean old cache crates
  -z, --old-orphan             Clean crates which are both old and orphan
  -x, --orphan                 Clean orphan cache crates i.e all crates which are not present in lock file generated till now
  -q, --query                  Return size of different .cargo/cache folders
      --scan-hidden-folder     Scan hidden folder for current command [env: TRIM_SCAN_HIDDEN_FOLDER=]
      --scan-target-folder     Scan target folder for current command [env: TRIM_SCAN_TARGET_FOLDER=]
  -t, --top <TOP>              Show certain number of top crates which have highest size
  -u, --update                 Update Cargo.lock file present inside config directory folder path
  -w, --wipe <WIPE>            Wipe folder [possible values: git, checkouts, db, registry, cache, index, index-cache, src]
  -h, --help                   Print help
  -V, --version                Print version
```

### Configuration

`cargo-trim` stores its configuration in a cargo_trim_config.toml file, located in the OS's default config directory.
The file contains various settings for scanning projects and managing crates.

To find the configuration file's location, use:
```
cargo trim config --location
```
Here are some configuration options and their default values:

#### 1. directory

**default: []**

**env: TRIM_DIRECTORY**

List of directory to scan for a Rust projects

#### 2. **ignore_file_name**

**default: []**

**env: TRIM_IGNORE**

List of file or directory names to ignore during scanning.
This is useful for excluding folders like `node_modules` or `venv` to enhance performance.

#### 3. **scan_hidden_folder**

**default: false**

**env: TRIM_SCAN_HIDDEN_FOLDER**

Indicates whether hidden folders (those starting with a dot) should be scanned.

#### 4. **scan_target_folder**

**default: false**

**env: TRIM_SCAN_TARGET_FOLDER**

Indicates whether the target folder should be scanned. The target folder name is determined by the `CARGO_BUILD_TARGET_DIR`, `CARGO_TARGET_DIR` environment variables, or defaults to `target`.

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg?style=for-the-badge
[license_link]: LICENSE
[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg?style=for-the-badge
[cratesio_link]: https://crates.io/crates/cargo-trim
