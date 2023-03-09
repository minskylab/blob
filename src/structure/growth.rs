use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha2::{Digest, Sha256};
use std::{fs, io, io::copy};
use tokio::fs::read_to_string;

use crate::llm::{engine::LLMEngine, templates::interpretation_prompt_template};

use super::software::{Project, SourceAtom};

#[derive(Debug, Clone)]
pub struct Growth {}

#[derive(Debug, Clone)]
pub struct BlobProcessedDir<T>
where
    T: Clone,
{
    // atom: SourceAtom<String>,
    pub root: PathBuf,
    pub level: usize,
    pub children: Vec<SourceAtom<T>>,
}

#[derive(Debug, Clone)]
pub struct ProcessFileResult {
    pub llm_response: String,
    pub file_path: PathBuf,
    pub hash: String,
}

impl Growth {
    pub fn new() -> Self {
        Growth {}
    }

    pub async fn traversal_modules(
        &mut self,
        mut software_project: Project,
    ) -> Vec<BlobProcessedDir<String>> {
        let mut source_file_map: Box<HashMap<String, String>> = Box::new(HashMap::new());

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

        let total_files = data.iter().fold(0, |acc, v| acc + v.children.len());
        let total_dirs = data.len();
        let max_level = data.iter().fold(0, |acc, v| acc.max(v.level));

        println!("Total files: {}", total_files);
        println!("Total dirs: {}", total_dirs);
        println!("Max level: {}", max_level);

        data
    }

    pub async fn process_file(
        &self,
        child: PathBuf,
        arc_engine_clone: Arc<LLMEngine>,
    ) -> Option<ProcessFileResult> {
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
            .completions_call(final_prompt.clone(), Some(vec!["#".to_string()]))
            .await;

        let (interpretation, error) = match completion_response.as_ref() {
            Ok(completion) => (
                Some(completion.choices.first().unwrap().text.trim().to_string()),
                None,
            ),
            Err(e) => {
                println!("Error: {}", e);
                (None, Some(e.to_string()))
            }
        };

        if let Some(err) = error {
            println!("Error: {}", err);
        }

        let hash = Self::calculate_file_hash(child.clone()).await;

        Some(ProcessFileResult {
            llm_response: interpretation.unwrap(),
            file_path: child.clone(),
            hash,
        })
    }

    pub async fn calculate_file_hash(file: PathBuf) -> String {
        let mut hasher = Sha256::new();
        let mut file = fs::File::open(file).unwrap();

        io::copy(&mut file, &mut hasher).unwrap();

        let hash_bytes = hasher.finalize();

        format!("{:x}", hash_bytes)
    }
}
