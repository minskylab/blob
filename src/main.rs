use std::path::PathBuf;
use std::process::Command;

use blob::context::BlobContextProcessor;
use blob::mutation::{ProjectMutationDraft, SourceFileMutation, SourceFileMutationDraft};
use clap::Parser;
use cli::tool::{BlobTool, Commands};
use dotenv::dotenv;
use llm::engine::LLMEngine;

use crate::structure::software::{Project, SourceAtomTyped};

mod blob;
mod codex;
mod llm_engine;
mod representation;
mod tool;
mod transformer;

#[tokio::main]

async fn main() {
    let cli = BlobTool::parse();

    let mut engine = LLMEngine::new();

    match &cli.command {
        Commands::Init { path, instruction } => {
            println!("Applying edits to {:?}", path);

            // let path_str = path.as_ref().unwrap();

            // let p = CodexProcessor::new(access_token);

            // for entry in glob(path_str).expect("Failed to read glob pattern") {
            //     match entry {
            //         Ok(path) => {
            //             // println!("{:?}", path.display());
            //             let path_str = Box::new(path.to_str().unwrap());

            //             let content = std::fs::read_to_string(*path_str).unwrap();
            //             let edit = p
            //                 .clone()
            //                 .codex_edit_call(content, instruction.as_ref().unwrap())
            //                 .await
            //                 .unwrap();

                let context_lines = definitions
                    .iter()
                    .map(|def| def.definition.clone())
                    .collect();

            //             std::fs::create_dir_all(prefix).unwrap();

            //             let file = std::fs::File::create(path).unwrap();
            //             let mut writer = std::io::BufWriter::new(file);

            //             writer
            //                 .write_all(edit.choices.first().unwrap().text.as_bytes())
            //                 .unwrap();
            //         }
            //         Err(e) => println!("{:?}", e),
            //     }
            // }
        }
        Commands::Analyze { file } => {
            // let definitions =
            // context_processor.retrieve_definitions(blob::context::BlobDefinitionKind::Project);

        Commands::Do { path, instruction } => {
            // Snapshot::new_full
            // let mutation = ;
            // let a = Box::new(mutation);

            let mutation = Box::new(MutationInit::new(
                path.clone().unwrap(),
                instruction.clone().unwrap(),
            ));

            let mut software_project = Project::new(PathBuf::from(project_root_path));

            let data = software_project
                .calculate_source(move |atom| {
                    // println!("Atom: {:?}", atom);

                    "".to_string()
                })
                .await
                .iter()
                .map(|a| match a {
                    SourceAtomTyped::Dir(_, children, _) => children.clone(),
                    _ => Vec::<SourceAtomTyped<String>>::new(),
                })
                .collect::<Vec<Vec<SourceAtomTyped<String>>>>();

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
