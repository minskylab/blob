use std::fmt::Display;

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm::engine::LLMEngine;

use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSlice;
use structure::growth::{DigestedSource, Growth, ProcessFileResult};
use tokio::spawn;
use tokio::time::sleep;

use crate::structure::software::{Project, Source};
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

    matches!(input.trim(), "y" | "yes")
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
                            source_file_mutation.parent.file_path
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
        Commands::Analyze { file: _ } => {
            let software_project = Project::new(PathBuf::from(project_root_path));

            let sorted_dirs = Growth::traversal_modules(software_project).await;

            let all_files = Growth::extract_all_files_from_digested_source(sorted_dirs).await;

            let arc_engine = Arc::new(engine);

            all_files
                .par_iter()
                .filter(|f| {
                    if let Some(mime_type) = (*f).payload().mime_type() {
                        mime_type.starts_with("text/")
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>()
                .chunks(4)
                .for_each(|sources| {
                    let processed_sources = sources
                        .into_par_iter()
                        .map(|source| async {
                            println!("Source {:?}", source.path());
                            let a = process_file((*source).clone(), Arc::clone(&arc_engine)).await;
                            "".to_string()
                        })
                        .collect::<Vec<_>>();

                    // println!("Processing {:?}\n", sources);
                });

            // for f in all_files {
            //     println!("{:?}", f);
            // }

            // for d in sorted_dirs {
            //     match d {
            //         DigestedSource::DigestedDir {
            //             root,
            //             level,
            //             children,
            //         } => {
            //             println!("{} {:?}", level, root);
            //             for child in children {
            //                 match child {
            //                     Source::File { path, payload } => {
            //                         println!("\t{:?} {:?}", path, payload);
            //                     }
            //                     _ => {}
            //                 }
            //             }
            //         }
            //         _ => {}
            //     }
            // }

            // let arc_engine = Arc::new(engine);

            // let workers_number = 2;

            // let pool = rayon::ThreadPoolBuilder::new()
            //     .num_threads(workers_number)
            //     .build()
            //     .unwrap();

            // for directory in data.take(1) {
            // let mut file_results: Vec<_> = Vec::new();

            // let children = directory.children.clone();
            // println!("{:?}", directory);
            // let chunks = children.chunks(workers_number);

            // children
            //     .par_chunks(workers_number)
            //     .map(|child_chunk| async {
            //         let child_chunk = child_chunk.iter().collect::<Vec<_>>();

            //         for child in child_chunk {
            //             let res = process_file(child.clone(), arc_engine.clone()).await;
            //             // process_file(child.clone(), arc_engine.clone()).await;
            //             // file_results.push(res.unwrap());

            //             // file_results.len()
            //         }
            //     })
            //     .collect();

            // for child in children {
            //     let r = pool
            //         .install(|| async {
            //             let res = process_file(child.clone(), arc_engine.clone()).await;
            //             // process_file(child.clone(), arc_engine.clone()).await;
            //             file_results.push(res.unwrap());

            //             file_results.len()
            //         })
            //         .await;

            //     println!("{}", r);
            // }
            // }

            // for directory in data {
            // spawn(async move {
            // let mut file_results = Vec::new();

            // let children = directory.children.clone();
            // let chunks = children.chunks(workers_number);

            // for child_chunk in chunks {
            //     let child_chunk = child_chunk.into_iter().collect::<Vec<_>>();

            //     for child in child_chunk {
            //         let res = process_file(child.clone(), arc_engine).await;
            //         file_results.push(res.unwrap());
            //     }
            // }

            // arc_engine;

            // let mut accumulated_results = Vec::new();
            // arc_engine.clone();
            // // let arc_engine_clone = Arc::clone(&arc_engine);
            // for res in file_results {
            //     let result = res; //.unwrap().unwrap();
            //     accumulated_results.push(result);
            // }

            // let dir_result = Growth::process_dir_results(
            //     directory.root,
            //     accumulated_results,
            //     arc_engine,
            // )
            // .await;

            // println!("Dir result: {:?}", "f");
            // });
            // }
        }
    }
}

async fn process_file<Payload>(
    source: Source<Payload>,
    arc_engine: Arc<LLMEngine>,
) -> Result<ProcessFileResult>
where
    Payload: Clone + Sync,
{
    // match child {
    //     SourceAtom::File(child, kind) => {
    //         let arc_engine_clone = Arc::clone(&arc_engine);

    //         println!("Processing {} - {}", child.to_str().unwrap(), kind);

    //         let interpretation = Growth::process_file(child.to_path_buf(), arc_engine_clone).await;
    //         println!("Interpretation: {:?}", interpretation);

    //         interpretation
    //     }
    //     _ => {
    //         println!("Unknown");

    //         Err(anyhow!("Source Atom is not a file"))
    //     }
    // }
    sleep(Duration::from_millis(100)).await;

    Ok(ProcessFileResult::default())
}
