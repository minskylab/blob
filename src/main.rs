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
use tokio::fs::read_to_string;

use crate::llm::templates::interpretation_prompt_template;
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

            let mut source_file_map: Box<HashMap<String, String>> = Box::new(HashMap::new());

            let mut software_project = Project::new(PathBuf::from(project_root_path));

            let mut data = software_project
                .calculate_source(move |atom| match atom {
                    SourceAtom::File(path, _) => {
                        let content = source_file_map.get(&path.to_str().unwrap().to_string());

                        match content {
                            Some(content) => content.to_owned(),
                            None => {
                                let kind = infer::get_from_path(path)
                                    .unwrap()
                                    .map(|v| v.mime_type().to_string())
                                    .unwrap_or(
                                        path.extension()
                                            .map(|v| {
                                                format!("text/{}", v.to_str().unwrap().to_string())
                                            })
                                            .unwrap_or("unknown/unknown".to_string())
                                            .to_string(),
                                    );

                                source_file_map
                                    .insert(path.to_str().unwrap().to_string(), kind.clone());

                                kind
                            }
                        }
                    }
                    _ => "".to_string(),
                })
                .await
                .iter()
                .map(|atom| match atom {
                    SourceAtom::Dir(path, children, _) => Some(BlobProcessedDir {
                        children: children.clone(),
                        level: path.to_str().to_owned().unwrap().split("/").count() - 1,
                        root: path.clone(),
                    }),
                    _ => None,
                })
                .filter(|atom| atom.is_some())
                .map(|atom| atom.unwrap())
                .collect::<Vec<BlobProcessedDir<String>>>();

            data.sort_by(|a, b| a.level.cmp(&b.level));
            data.reverse();

            // let pool = rayon::ThreadPoolBuilder::new()
            //     .num_threads(4)
            //     .build()
            //     .unwrap();

            let arc_engine = Arc::new(engine);

            let wg = WaitGroup::new();

            data[..1].iter().for_each(|directory| {
                for child in directory.children.clone() {
                    match child {
                        SourceAtom::File(child, kind) => {
                            let arc_engine_clone = Arc::clone(&arc_engine);

                            let wg = wg.clone();
                            tokio::spawn(async move {
                                println!("Processing {} - {}", child.to_str().unwrap(), kind);

                                let file_content = read_to_string(child.clone())
                                    .await
                                    .unwrap_or("".to_string());

                                let max_char = 10_000;

                                let upper = if max_char > file_content.len() {
                                    file_content.len()
                                } else {
                                    max_char
                                };

                                let final_prompt = interpretation_prompt_template(
                                    child.as_path(),
                                    file_content.get(..upper).unwrap().to_string(),
                                "Please generate a comprehensive, detailed, and specific summary of the following code snippet. Your summary should include the following information:

1. Purpose of the code: what does the code do, what problem does it solve, and what is its intended effect in the context of the overall system or business logic? Provide an overview of the code's main function and any notable behavior.
2. Programming constructs used: what programming language is the code written in, and what specific constructs are used (e.g. functions, classes, loops, conditionals, etc.)? Describe the syntax, purpose, and behavior of the constructs used.
3. Algorithms or data structures employed: does the code use any specific algorithms or data structures (e.g. sorting algorithms, tree structures, etc.)? If so, explain what they are and how they are used in the code. Discuss the time and space complexity of any algorithms used.
4. Business logic inferred from the code: what can you infer about the business logic or system the code is a part of based on the code itself? Provide examples of the inputs, outputs, and processing of the code that support your inference.
5. Notable features or challenges: are there any interesting or challenging aspects of the code that you would like to highlight? This can include efficiency, scalability, maintainability, edge cases, etc.

In your summary, please explicitly state any assumptions or contextual information necessary to understand the code and its behavior within the larger system. Additionally, use appropriate references to any external dependencies, data sources, or other related code snippets as needed.
".to_string());

                                let completion_response = arc_engine_clone
                                    .codex_processor
                                    .clone()
                                    .completions_call(
                                        final_prompt.clone(),
                                        Some(vec!["#".to_string()]),
                                    )
                                    .await;

                                let (interpretation, error) = match completion_response.as_ref() {
                                    Ok(completion) => (
                                        Some(
                                            completion
                                                .choices
                                                .first()
                                                .unwrap()
                                                .text
                                                .trim()
                                                .to_string(),
                                        ),
                                        None,
                                    ),
                                    Err(e) => {
                                        println!("Error: {}", e);
                                        (None, Some(e.to_string()))
                                    }
                                };

                                println!("{}", interpretation.unwrap());

                                drop(wg);
                            });
                        }
                        _ => println!("Unknown"),
                    };
                }

                println!("Processing {:}\n", directory.root.to_str().unwrap());
            });
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
            wg.wait();
        }
    }
}

#[derive(Debug)]
struct BlobProcessedDir<T>
where
    T: Clone,
{
    // atom: SourceAtom<String>,
    root: PathBuf,
    level: usize,
    children: Vec<SourceAtom<T>>,
}
