use std::process::Command;

use blob::analysis::ProjectAnalysisDraft;
use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm::engine::LLMEngine;

mod blob;
mod cli;
mod codex;
mod llm;
mod representation;

fn ask_for_confirmation() -> bool {
    println!("Do you want to apply this mutation? (y/N):");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "y" | "yes" => true,
        _ => false,
    }
}

fn apply_source_file_mutation(
    mutation_folder_path: String,
    source_file_mutation: SourceFileMutation,
) {
    let res = Command::new("cp")
        .arg("-r")
        .arg(mutation_folder_path)
        .arg(source_file_mutation.parent.file_path)
        .output()
        .unwrap();

    let output = String::from_utf8_lossy(&res.stdout);

    println!("{}", output);
}

fn apply_mutation_script(mutation_script_path: String) {
    let res = Command::new("bash")
        .arg(mutation_script_path)
        .output()
        .unwrap();
    let output = String::from_utf8_lossy(&res.stdout);

    println!("{}", output);
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = BlobTool::parse();

    let project_root_path = cli.root.unwrap_or(".".to_string());

    let mut engine = LLMEngine::new();
    let context_processor = BlobContextProcessor::new(project_root_path.clone());

    match &cli.command {
        Commands::Do {
            instruction,
            file,
            yes: _,
        } => match file {
            Some(file) => {
                let mutation_draft = Box::new(SourceFileMutationDraft::new(
                    file.clone(),
                    instruction.clone().unwrap(),
                ));

                let source_file_mutation = engine.transform_specific_file(mutation_draft).await;

                let mutation_folder_path =
                    context_processor.save_source_file_mutation(source_file_mutation.clone());

                println!("Mutation saved into {mutation_folder_path}");

                match ask_for_confirmation() {
                    true => {
                        println!(
                            "Updated source file to {}",
                            source_file_mutation.clone().parent.file_path
                        );
                        apply_source_file_mutation(mutation_folder_path, source_file_mutation)
                    }
                    false => println!("Mutation discarded."),
                }
            }
            None => {
                let definitions = context_processor
                    .retrieve_definitions(blob::context::BlobDefinitionKind::Project);

                // let mut self_definitions = context_processor
                //     .retrieve_definitions(blob::context::BlobDefinitionKind::SelfReference);

                // definitions.append(&mut self_definitions);

                let context_lines = definitions
                    .iter()
                    .map(|def| def.definition.clone())
                    .collect();

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

                // match yes.unwrap_or(ask_for_confirmation()) {
                match ask_for_confirmation() {
                    true => {
                        println!("Applying edits to {}", project_root_path.clone());
                        apply_mutation_script(script_path);
                    }
                    false => println!("Mutation discarded."),
                }
            }
        },
        Commands::Define { definition } => {
            context_processor.save_project_definitions(vec![definition.clone().unwrap()]);
        }
        Commands::Analyze { file } => {
            // let definitions =
            //     context_processor.retrieve_definitions(blob::context::BlobDefinitionKind::Project);

            // let context_lines = definitions
            //     .iter()
            //     .map(|def| def.definition.clone())
            //     .collect();

            // let analysis = engine.analyze_project(context_lines).await;

            let analysis = ProjectAnalysisDraft::new(
                project_root_path.clone(),
                "Please generate a comprehensive, detailed, and specific summary of the following code snippet. Your summary should include the following information:
                
                1. Purpose of the code: what does the code do, what problem does it solve, and what is its intended effect in the context of the overall system or business logic? Provide an overview of the code's main function and any notable behavior.
                2. Programming constructs used: what programming language is the code written in, and what specific constructs are used (e.g. functions, classes, loops, conditionals, etc.)? Describe the syntax, purpose, and behavior of the constructs used.
                3. Algorithms or data structures employed: does the code use any specific algorithms or data structures (e.g. sorting algorithms, tree structures, etc.)? If so, explain what they are and how they are used in the code. Discuss the time and space complexity of any algorithms used.
                4. Business logic inferred from the code: what can you infer about the business logic or system the code is a part of based on the code itself? Provide examples of the inputs, outputs, and processing of the code that support your inference.
                5. Notable features or challenges: are there any interesting or challenging aspects of the code that you would like to highlight? This can include efficiency, scalability, maintainability, edge cases, etc.
                ".to_string(),
                // file.clone().unwrap(),
                // definitions,
            );

            engine.generate_recursive_analysis(analysis.clone()).await;

            println!("Analysis: {:#?}", analysis);
        }
    }
}
