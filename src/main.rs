use clap::Parser;
use tool::{BlobTool, Commands};

mod codex;
mod tool;

#[tokio::main]

async fn main() {
    let cli = BlobTool::parse();

    match &cli.command {
        Commands::Add { name } => {
            println!("Add command");
            println!("Name: {:?}", name);
        }
    }
}
