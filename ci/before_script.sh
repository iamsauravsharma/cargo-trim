#!/usr/bin/env bash/
set -ex

rustfmt_and_clippy_latest_nightly() {
  if [[ "$RUSTFMT_ADDED" == "false" ]]
  then
    LAST_AVAILABLE_FMT=$(curl https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/rustfmt)
    rustup toolchain install nightly-"${LAST_AVAILABLE_FMT}"
    rustup component add rustfmt --toolchain nightly-"${LAST_AVAILABLE_FMT}"
    rustup default nightly
  fi
  if [[ "$CLIPPY_ADDED" == "false" ]]
  then
    LAST_AVAILABLE_CLIPPY=$(curl https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin/clippy)
    rustup toolchain install nightly-"${LAST_AVAILABLE_CLIPPY}"
    rustup component add clippy --toolchain nightly-"${LAST_AVAILABLE_CLIPPY}"
    rustup default nightly
  fi
}

print_out_version(){
  rustup --version
  rustc --version
  cargo --version
  if [[ $RUSTFMT_ADDED == "false" ]]
  then
    cargo +nightly-"${LAST_AVAILABLE_FMT}" fmt --version
  else
    cargo fmt --version
  fi
  if [[ $CLIPPY_ADDED == "false" ]]
  then
    cargo +nightly-"${LAST_AVAILABLE_CLIPPY}" clippy --version
  else
    cargo clippy --version
  fi
  export LAST_AVAILABLE_FMT
  export LAST_AVAILABLE_CLIPPY
}

target_print_out_list() {
  rustup --version
  rustc --version
  cargo --version
  rustup target list --installed
}

if [[ -z $TARGET ]]
then
  rustfmt_and_clippy_latest_nightly
  print_out_version
else
  target_print_out_list
fi

set +x