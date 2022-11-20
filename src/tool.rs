use std::str;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(propagate_version = true)]
#[command(author, version, about, long_about)]
pub struct BlobTool {
    // /// Name of the person to greet
    // #[arg(short, long)]
    // pub name: String,

    // /// Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // pub count: u8,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new blob
    Add { name: Option<String> },
}
