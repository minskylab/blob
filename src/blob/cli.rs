use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(propagate_version = true)]
#[command(author, version, about, long_about)]
pub struct BlobTool {
    #[arg(short, long)]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Do an edit on a blob project through the OpenAI Codex API.
    Do { instruction: Option<String> },

    /// Give a definition related to the project, util to increase the quality of the model predictions.
    Define { definition: Option<String> },
}
