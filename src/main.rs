use clap::Parser;
use codex::Processor;
use tool::{BlobTool, Commands};

mod codex;
mod tool;

#[tokio::main]

async fn main() {
    let cli = BlobTool::parse();

    let access_token = std::env::var("OPENAI_API_KEY").unwrap();

    let p = Processor::new(access_token);

    match &cli.command {
        Commands::Apply { path, instruction } => {
            println!("Applying edits to {:?}", path);

            let content = std::fs::read_to_string(path.as_ref().unwrap()).unwrap();

            let resp = p.codex_call(content, instruction.as_ref().unwrap()).await;

            println!("{:?}", resp);
        }
    }
}
