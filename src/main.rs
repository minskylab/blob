use std::process::Command;

use blob::blob::BlobTemporalContextProcessor;
use clap::Parser;
use dotenv::dotenv;
use llm_engine::performer::LLMEngine;
use tool::tool::{BlobTool, Commands};
use transformer::mutation::ProjectMutationDraft;

mod blob;
mod codex;
mod llm_engine;
mod representation;
mod tool;
mod transformer;

#[tokio::main]

async fn main() {
    dotenv().ok();

    let cli = BlobTool::parse();

    let mut engine = LLMEngine::new();
    let context_processor = BlobTemporalContextProcessor::new();

    match &cli.command {
        Commands::Init { path, instruction } => {
            println!("Applying edits to {}", path.clone().unwrap());
        }

        Commands::Do { path, instruction } => {
            // Snapshot::new_full
            let mutation =
                ProjectMutationDraft::new(path.clone().unwrap(), instruction.clone().unwrap());

            let mutation_scripted = engine.generate_bash_script(Box::new(mutation)).await;

            println!("{}", mutation_scripted.parent.parent.created_at);

            let script_path = context_processor.save_new_context(mutation_scripted);

            println!("Script saved to {script_path}");

            print!("Do you want to apply this mutation? (y/n)\n>");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim() == "y" {
                println!("Applying edits to {}", path.clone().unwrap());

                let res = Command::new("bash").arg(script_path).output().unwrap();
                let output = String::from_utf8_lossy(&res.stdout);

                println!("{}", output);
            } else {
                println!("Mutation cancelled");
            }
        }
        Commands::Context { path, instruction } => {
            println!("Applying edits to {}", path.clone().unwrap());
        }
    }
}
