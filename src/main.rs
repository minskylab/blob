use clap::Parser;
use llm_engine::performer::LLMEngine;
use representation::filters::{FilterAggregate, GitignoreFilter};
use representation::tree::TreeIter;
use std::path::Path;
use tool::tool::{BlobTool, Commands};

mod blob;
mod codex;
mod llm_engine;
mod persistence;
mod representation;
mod tool;

#[tokio::main]

async fn main() {
    let cli = BlobTool::parse();

    let mut engine = LLMEngine::new();
    let mut filters = FilterAggregate::default();

    match &cli.command {
        Commands::Create { path, instruction } => {
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
            let path_str = path.as_ref().unwrap();
            let root = Path::new(path_str).to_owned();

            let github_filter = GitignoreFilter::new(root.clone()).unwrap().unwrap();

            filters.push(github_filter);

            let mut tree_iter = TreeIter::new(root, filters).unwrap();

            let snp = engine
                .generate_proposal(tree_iter.by_ref(), instruction.as_ref().unwrap().clone())
                .await;

            // // let path_root = String::from(root.to_str().unwrap().clone());

            // // let snp = Snapshot::new(
            // //     path_root,
            // //     current_structure,
            // //     proposed_structure,
            // //     instruction.as_ref().unwrap().clone(),
            // // );

            let bash_guide = snp.generate_prompt().unwrap();

            // println!("{bash_guide}");

            let completed_bash = engine
                .generate_transformer(tree_iter.by_ref(), bash_guide)
                .await;

            println!("{completed_bash}");
            // snp.

            // println!("{current_dir}");
            // println!("{current_structure}");
            // println!("{proposed_structure}");

            // println!("Planning edits to {:?}", path);

            // let tree = processor.construct(&mut tree_iter).unwrap();

            // println!("{tree}");
        }
    }
}
