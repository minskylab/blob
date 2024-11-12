use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use blob::blob::context::{BlobContextProcessor, BlobDefinitionKind};
use blob::blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use blob::cli::tool::{BlobTool, Commands};
use blob::llm::engine::LLMEngine;
use clap::Parser;
use dotenv::dotenv;

use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use blob::structure::growth::{Growth, ProcessFileResult};

use tokio::time::sleep;

use blob::structure::software::{Project, Source};

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
                let definitions =
                    context_processor.retrieve_definitions(BlobDefinitionKind::Project);

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
