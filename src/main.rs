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
        Commands::Do { instruction } => {
            let defs = context_processor
                .retrieve_definitions(project_path.clone(), blob::blob::BlobDefinitionKind::User);

            let context_lines = defs.iter().map(|def| def.definition.clone()).collect();

            let mutation = ProjectMutationDraft::new(
                project_path.clone(),
                instruction.clone().unwrap(),
                context_lines,
            );

            let mutation_scripted = engine.generate_bash_script(Box::new(mutation)).await;

            let script_path = context_processor.save_new_context(mutation_scripted.clone());

            println!(
                "Predicted commands:\n{}\n",
                mutation_scripted.predicted_commands,
            );

            println!("Script saved to {script_path}");

            println!("Do you want to apply this mutation? (y/N):");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "y" | "yes" => {
                    println!("Applying edits to {}", project_path.clone());

                    let res = Command::new("bash").arg(script_path).output().unwrap();
                    let output = String::from_utf8_lossy(&res.stdout);

                    println!("{}", output);
                }
                _ => println!("Mutation discarded."),
            }
        }
        Commands::Define { definition } => {
            context_processor.save_new_definition(
                project_path.clone(),
                blob::blob::BlobDefinitionKind::User,
                definition.clone().unwrap(),
            );
        }
    }
}
