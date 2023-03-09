use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm::engine::LLMEngine;
use structure::growth::Growth;
use tokio::fs::read_to_string;

use crate::llm::templates::interpretation_prompt_template;
use crate::structure::growth::ProcessDirResult;
use crate::structure::software::{Project, SourceAtom};
use crossbeam_utils::sync::WaitGroup;

mod blob;
mod cli;
mod codex;
mod llm;
mod representation;
pub mod structure;

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
            // context_processor.retrieve_definitions(blob::context::BlobDefinitionKind::Project);

            // let context_lines = definitions
            //     .iter()
            //     .map(|def| def.definition.clone())
            //     .collect();

            // let analysis = engine.analyze_project(context_lines).await;

            // let mut source_file_map: Box<HashMap<String, String>> = Box::new(HashMap::new());

            let software_project = Project::new(PathBuf::from(project_root_path));

            let data = Growth::new()
                .traversal_modules(software_project)
                .await
                .to_owned()
                .into_iter();

            let arc_engine = Arc::new(engine);

            let workers_number = 4;

            for directory in data.take(2) {
                let wg = WaitGroup::new();
                let mut file_results = Vec::new();

                for child in directory.children {
                    match child {
                        SourceAtom::File(child, kind) => {
                            let arc_engine_clone = Arc::clone(&arc_engine);

                            let wg = wg.clone();
                            file_results.push(tokio::spawn(async move {
                                println!("Processing {} - {}", child.to_str().unwrap(), kind);

                                let interpretation =
                                    Growth::new().process_file(child, arc_engine_clone).await;
                                println!("Interpretation: {:?}", interpretation);

                                // file_results.push(interpretation.unwrap());
                                drop(wg);

                                interpretation
                            }));
                        }
                        _ => println!("Unknown"),
                    };
                }
                wg.wait();

                let mut accumulated_results = Vec::new();

                let arc_engine_clone = Arc::clone(&arc_engine);
                for res in file_results {
                    let result = res.await.unwrap().unwrap();
                    accumulated_results.push(result);
                }

                // let dir_result = ProcessDirResult {
                //     dir_path: directory.root,
                //     processed_files: accumulated_results,
                // };
                let dir_result = Growth::process_dir_results(
                    directory.root,
                    accumulated_results,
                    arc_engine_clone,
                )
                .await;

                // file_results
                //     .iter()
                //     .map(|res| async move { res.await.unwrap().unwrap() });

                println!("Dir result: {:?}", dir_result);

                // println!("Processing {:}\n", directory.root.to_str().unwrap());
            }

            // for directory in data.iter().take(1) {
            //     // let children_list = directory.children.clone().iter();

            // }

            // let analysis = ProjectAnalysisDraft::new_with_default_prompt(project_root_path.clone());

            // let result = engine.generate_recursive_analysis(Box::new(analysis)).await;

            // // println!("Analysis: {:#?}", analysis);
            // let document_content = result
            //     .source_files
            //     .iter()
            //     .filter(|source| source.error.is_none())
            //     .map(|source| {
            //         format!(
            //             "## {}\n### Definition\n{}",
            //             source.file_path,
            //             source.result.as_ref().unwrap()
            //         )
            //     })
            //     .collect::<Vec<String>>()
            //     .join("\n\n");

            // // save document content to file
            // let mut file = File::create("analysis_full.md").unwrap();
            // file.write_all(document_content.as_bytes()).unwrap();
        }
    }
}
