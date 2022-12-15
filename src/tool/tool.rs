use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(propagate_version = true)]
#[command(author, version, about, long_about)]
pub struct BlobTool {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new blob project (only a context for your edits)
    Create {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        instruction: Option<String>,
    },

    /// Do an edit on a blob project through the OpenAI Codex API
    Do {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        instruction: Option<String>,
    },
}