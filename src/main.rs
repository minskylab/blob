use std::io::Write;

use clap::Parser;
use codex::Processor;
use filters::{FilterAggregate, GitignoreFilter};
use tool::{BlobTool, Commands};

use glob::glob;
use std::path::Path;

use print::PrintProcessorBuilder;
use tree::TreeProcessor;

mod blob;
mod codex;
mod codex_responses;
mod filters;
mod print;
mod tool;
mod tree;

#[tokio::main]

async fn main() {
    let cli = BlobTool::parse();

    let access_token = std::env::var("OPENAI_API_KEY").unwrap();

    match &cli.command {
        Commands::Apply { path, instruction } => {
            println!("Applying edits to {:?}", path);

            let path_str = path.as_ref().unwrap();

            let p = Processor::new(access_token);

            for entry in glob(path_str).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        // println!("{:?}", path.display());
                        let path_str = Box::new(path.to_str().unwrap());

                        let content = std::fs::read_to_string(*path_str).unwrap();
                        let edit = p
                            .clone()
                            .codex_edit_call(content, instruction.as_ref().unwrap())
                            .await
                            .unwrap();

                        // println!("\n{edit}");

                        let new_path = format!("_blobs/{}", *path_str);
                        let path = Path::new(new_path.as_str());
                        let prefix = path.parent().unwrap();

                        println!("Writing to {:?}", new_path);

                        std::fs::create_dir_all(prefix).unwrap();

                        let file = std::fs::File::create(path).unwrap();
                        let mut writer = std::io::BufWriter::new(file);

                        writer
                            .write_all(edit.choices.first().unwrap().text.as_bytes())
                            .unwrap();
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }

        Commands::Plan { path, instruction } => {
            let mut filters = FilterAggregate::default();

            let dir = Path::new(path.as_ref().unwrap());
            let processor = PrintProcessorBuilder::new(From::from(dir));

            let github_filter = GitignoreFilter::new(dir).unwrap().unwrap();

            filters.push(github_filter);

            let mut tree_iter = tree::TreeIter::new(dir, filters).unwrap();

            println!("Planning edits to {:?}", path);

            processor.build().process(&mut tree_iter).unwrap();
        }
    }
}
