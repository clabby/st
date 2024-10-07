set positional-arguments
alias t := test
alias l := lint
alias f := fmt-fix
alias b := build
alias h := hack

# default recipe to display help information
default:
  @just --list

# Install the `st` binary
install:
  cargo install --bin st --path . --force

# Build for the native target
build:
  cargo build --workspace $@

# Test for the native target with all features. By default, excludes online tests.
test *args="":
  cargo nextest run --workspace --all --all-features {{args}}

# Fixes the formatting of the workspace
fmt-fix:
  cargo +nightly fmt --all

# Check the formatting of the workspace
fmt-check:
  cargo +nightly fmt --all -- --check

# Lint the workspace
lint: fmt-check lint-docs
  cargo +nightly clippy --workspace --all --all-features --all-targets -- -D warnings

# Runs `cargo hack check` against the workspace
hack:
  cargo hack check --feature-powerset --no-dev-deps

# Lint the Rust documentation
lint-docs:
  RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps --document-private-items 
