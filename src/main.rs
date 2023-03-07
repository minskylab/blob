use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm::engine::LLMEngine;

use crate::structure::software::{Project, SourceAtom};

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

            let mut source_file_map: Box<HashMap<String, String>> = Box::new(HashMap::new());

            let mut software_project = Project::new(PathBuf::from(project_root_path));

            let data = software_project
                .calculate_source(move |atom| {
                    println!("Atom: {:?}", atom);

                    match atom {
                        SourceAtom::File(path, _) => {
                            let content = source_file_map.get(&path.to_str().unwrap().to_string());

                            match content {
                                Some(content) => content.to_owned(),
                                None => {
                                    println!("Reading file: {:?}", path);

                                    let kind = infer::get_from_path(path)
                                        .unwrap()
                                        .unwrap()
                                        .mime_type()
                                        .to_string();
                                    // .unwrap()
                                    // .expect("file read successfully")
                                    // .expect("file type is known")
                                    // .to_string();

                                    // let content = std::fs::read_to_string(path).unwrap();

                                    source_file_map
                                        .insert(path.to_str().unwrap().to_string(), kind.clone());

                                    kind
                                }
                            }
                        }
                        _ => "".to_string(),
                    }
                })
                .await
                .iter()
                .map(|a| match a {
                    SourceAtom::Dir(_, children, _) => children.clone(),
                    _ => Vec::<SourceAtom<String>>::new(),
                })
                .collect::<Vec<Vec<SourceAtom<String>>>>();

            println!("Data: {:?}", data);

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
