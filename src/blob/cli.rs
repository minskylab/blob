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
    /// Init a new blob project (only a context for your edits)
    Init {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        instruction: Option<String>,
    },

    /// Do an edit on a blob project through the OpenAI Codex API
    Do {
        // #[arg(short, long)]
        instruction: Option<String>,
    },

    Context {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        instruction: Option<String>,
    },
}
