#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use clap::Parser;

mod cli;
mod config;
mod constants;
mod ctx;
mod errors;
mod git;
mod subcommands;
mod tree;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::Cli::parse().run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
