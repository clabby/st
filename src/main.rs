#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::Result;
use clap::Parser;

mod cli;
mod constants;
mod ctx;
mod git;
mod tree;
mod subcommands;

#[tokio::main]
async fn main() -> Result<()> {
    cli::Cli::parse().run().await
}
