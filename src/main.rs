use std::process::Command;

use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm_engine::performer::LLMEngine;

mod blob;
mod cli;
mod codex;
mod llm_engine;
mod representation;

#[tokio::main]

async fn main() {
    dotenv().ok();

    let cli = BlobTool::parse();

    let project_root_path = cli.root.unwrap_or(".".to_string());

    let mut engine = LLMEngine::new();
    let context_processor = BlobContextProcessor::new(project_root_path.clone());

    match &cli.command {
        Commands::Do { instruction, file } => match file {
            Some(file) => {
                let mutation_draft = Box::new(SourceFileMutationDraft::new(
                    file.clone(),
                    instruction.clone().unwrap(),
                ));

                let source_file_mutation = engine.transform_specific_file(mutation_draft).await;

                let mutation_folder_path =
                    context_processor.save_source_file_mutation(source_file_mutation.clone());

                println!("Mutation saved into {mutation_folder_path}");

                println!("Do you want to apply this mutation? (y/N):");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                match input.trim() {
                    "y" | "yes" => {
                        let res = Command::new("mv")
                            .arg(mutation_folder_path)
                            .arg(source_file_mutation.parent.file_path)
                            .output()
                            .unwrap();

                        let output = String::from_utf8_lossy(&res.stdout);

                        println!("{}", output);
                    }
                    _ => println!("Mutation discarded."),
                }
            }
            None => {
                let defs =
                    context_processor.retrieve_definitions(blob::context::BlobDefinitionKind::User);

                let context_lines = defs.iter().map(|def| def.definition.clone()).collect();

                let mutation = ProjectMutationDraft::new(
                    project_root_path.clone(),
                    instruction.clone().unwrap(),
                    context_lines,
                );

                let mutation_scripted = engine.generate_project_mutation(Box::new(mutation)).await;

                let script_path =
                    context_processor.save_project_mutation(mutation_scripted.clone());

                println!(
                    "Predicted commands:\n{}\n",
                    mutation_scripted.predicted_commands,
                );

                println!("Script saved into {script_path}");

                println!("Do you want to apply this mutation? (y/N):");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                match input.trim() {
                    "y" | "yes" => {
                        println!("Applying edits to {}", project_root_path.clone());

                        let res = Command::new("bash").arg(script_path).output().unwrap();
                        let output = String::from_utf8_lossy(&res.stdout);

                        println!("{}", output);
                    }
                    _ => println!("Mutation discarded."),
                }
            }
        },
        Commands::Define { definition } => {
            context_processor.save_project_definition(
                blob::context::BlobDefinitionKind::User,
                definition.clone().unwrap(),
            );
        }
    }
}
