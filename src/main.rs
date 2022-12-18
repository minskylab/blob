use std::process::Command;

use blob::blob::BlobContextProcessor;
use blob::cli::{BlobTool, Commands};
use clap::Parser;
use dotenv::dotenv;
use llm_engine::performer::LLMEngine;
use transformer::mutation::ProjectMutationDraft;

mod blob;
mod codex;
mod llm_engine;
mod representation;
mod transformer;

#[tokio::main]

async fn main() {
    dotenv().ok();

    let cli = BlobTool::parse();

    let mut engine = LLMEngine::new();
    let context_processor = BlobContextProcessor::new();

    let project_path = cli.path.unwrap_or(".".to_string());

    match &cli.command {
        Commands::Init { path, instruction } => {
            println!("Applying edits to {}", path.clone().unwrap());
        }

        Commands::Do { instruction } => {
            let mutation =
                ProjectMutationDraft::new(project_path.clone(), instruction.clone().unwrap());

            let mutation_scripted = engine.generate_bash_script(Box::new(mutation)).await;

            println!("{}", mutation_scripted.parent.parent.created_at);

            let script_path = context_processor.save_new_context(mutation_scripted.clone());

            println!(
                "Predicted commands:\n{}",
                mutation_scripted.predicted_commands,
            );
            println!("Script saved to {script_path}");

            print!("Do you want to apply this mutation? (y/n)\n>");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim() == "y" {
                println!("Applying edits to {}", project_path.clone());

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
