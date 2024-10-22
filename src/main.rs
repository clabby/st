#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use clap::Parser;
use errors::StResult;

mod cli;
mod config;
mod constants;
mod ctx;
mod errors;
mod git;
mod subcommands;
mod tree;

#[tokio::main]
async fn main() -> StResult<()> {
    cli::Cli::parse().run().await
}
