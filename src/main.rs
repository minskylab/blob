use std::process::Command;

use blob::blob::BlobTemporalContextProcessor;
use clap::Parser;
use dotenv::dotenv;
use llm_engine::performer::LLMEngine;
use tool::tool::{BlobTool, Commands};
use transformer::mutation::ProjectMutationDraft;

mod blob;
mod codex;
mod llm_engine;
mod representation;
mod tool;
mod transformer;

#[tokio::main]

async fn main() {
    dotenv().ok();

    let cli = BlobTool::parse();

    let mut engine = LLMEngine::new();
    let context_processor = BlobTemporalContextProcessor::new();

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

            //             // println!("\n{edit}");

            //             let new_path = format!("_blobs/{}", *path_str);
            //             let path = Path::new(new_path.as_str());
            //             let prefix = path.parent().unwrap();

            //             println!("Writing to {:?}", new_path);

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

        Commands::Do { path, instruction } => {
            // Snapshot::new_full
            let mutation =
                ProjectMutationDraft::new(path.clone().unwrap(), instruction.clone().unwrap());

            let mutation_scripted = engine.generate_bash_script(Box::new(mutation)).await;
            // let generated_script = mutation_scripted.bash_script;
            // let path_str = path.as_ref().unwrap();
            // let root = Path::new(path_str).to_owned();
            // let now: DateTime<Utc> = Utc::now();
            // println!("UTC now is: {}", now.to_rfc3339());

            println!("{}", mutation_scripted.parent.parent.created_at);

            let script_path = context_processor.save_new_context(mutation_scripted);

            println!("Script saved to {script_path}");

            print!("Do you want to apply this mutation? (y/n)\n>");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim() == "y" {
                println!("Applying edits to {}", path.clone().unwrap());
                // let mutation = context_processor
                //     .load_context(script_path)
                //     .unwrap()
                //     .unwrap();

                // let completed_bash = mutation.extend_with_llm(&mut engine).await;

                // let b = completed_bash.unwrap();
                let res = Command::new("bash").arg(script_path).output().unwrap();
                let output = String::from_utf8_lossy(&res.stdout);

                println!("{}", output);
            } else {
                println!("Mutation cancelled");
            }

            // let github_filter = GitignoreFilter::new(root.clone()).unwrap().unwrap();

            // filters.push(github_filter);

            // let mut tree_iter = TreeIter::new(root, filters).unwrap();

            // let snp = engine
            //     .generate_proposal(tree_iter.by_ref(), instruction.as_ref().unwrap().clone())
            //     .await;

            // // // let path_root = String::from(root.to_str().unwrap().clone());

            // // // let snp = Snapshot::new(
            // // //     path_root,
            // // //     current_structure,
            // // //     proposed_structure,
            // // //     instruction.as_ref().unwrap().clone(),
            // // // );

            // let bash_guide = snp.generate_prompt().unwrap();

            // // println!("{bash_guide}");

            // mutation.generate_proposal(&mut engine).await;

            // let b = mutation.generate_prompt().unwrap();

            // let completed_bash = mutation.extend_with_llm(&mut engine).await;

            // let b = completed_bash.unwrap();

            // println!("{generated_script}");
            // snp.

            // println!("{current_dir}");
            // println!("{current_structure}");
            // println!("{proposed_structure}");

            // println!("Planning edits to {:?}", path);

            // let tree = processor.construct(&mut tree_iter).unwrap();

            // println!("{tree}");
        }
        Commands::Context { path, instruction } => {
            println!("Applying edits to {:?}", path);
        }
    }
}
