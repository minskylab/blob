use std::borrow::BorrowMut;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::blob::analysis::{
    ProjectAnalysisDraft, ProjectAnalysisResult, ProjectSourceFileAnalysis,
};
use crate::blob::mutation::{
    ProjectMutation, ProjectMutationDraft, ProjectMutationProposed, SourceFileMutation,
    SourceFileMutationDraft,
};
use crate::codex::processor::CodexProcessor;
use crate::representation::tree::iterator::Event;
use crate::representation::tree::reader::TreeFileWalker;
use crate::representation::{
    tree::iterator::{TreeIter, TreeProcessor},
    tree::representation::TreeRepresentation,
};

pub struct LLMEngine {
    llm_representation: TreeRepresentation,
    // walker: TreeFileWalker,
    codex_processor: CodexProcessor,
}

impl LLMEngine {
    pub fn new() -> Self {
        let access_token = std::env::var("OPENAI_API_KEY").unwrap();

        LLMEngine {
            llm_representation: TreeRepresentation::new(),
            // walker: ,
            codex_processor: CodexProcessor::new(access_token),
        }
    }

    fn generate_context(&mut self, root: &mut TreeIter) -> String {
        self.llm_representation.construct(root).unwrap()
    }

    pub async fn generate_structure_proposal(
        &mut self,
        mut mutation_draft: Box<ProjectMutationDraft>,
    ) -> Box<ProjectMutationProposed> {
        let mut root_tree = mutation_draft.tree_iter();
        let context = self.generate_context(root_tree.as_mut());

        let prompt = mutation_draft.prompt.clone();

        let edit = self
            .codex_processor
            .clone()
            .edit_call(context.clone(), prompt.clone())
            .await
            .unwrap();

        Box::new(ProjectMutationProposed::new_from_parent(
            mutation_draft,
            context.clone(),
            edit.choices.first().unwrap().text.clone(),
        ))
    }

    pub async fn generate_project_mutation(
        &mut self,
        mutation_draft: Box<ProjectMutationDraft>,
    ) -> ProjectMutation {
        let snapshot = self.generate_structure_proposal(mutation_draft).await;

        let next_prompt = snapshot.clone().generate_prompt().unwrap();

        let completion = self
            .codex_processor
            .clone()
            .completions_call(next_prompt.clone(), None)
            .await
            .unwrap();

        let predicted_commands = completion.choices.first().unwrap().text.clone();

        let full_script = format!("{}{}", next_prompt, predicted_commands);

        ProjectMutation::new_from_parent(snapshot.clone(), predicted_commands, full_script)
    }

    pub async fn transform_specific_file(
        &mut self,
        mutation_draft: Box<SourceFileMutationDraft>,
    ) -> SourceFileMutation {
        // let mut root_tree = mutation_draft.tree_iter();
        // let context = self.generate_context(root_tree.as_mut());

        let prompt = mutation_draft.prompt.clone();
        // mutation_draft.
        // let file_path = format!("{}/{}", project_path.clone(), file.clone());
        let file_content = std::fs::read_to_string(mutation_draft.file_path.clone()).unwrap();

        let edit = self
            .codex_processor
            .clone()
            .edit_call(file_content.clone(), prompt)
            .await
            .unwrap();
        // self.generate_bash_script(Box::new(mutation_draft)).await

        SourceFileMutation::new_from_parent(
            mutation_draft,
            file_content,
            edit.choices.first().unwrap().text.clone(),
        )
    }

    pub async fn generate_recursive_analysis(
        &mut self,
        mut project_analysis_draft: Box<ProjectAnalysisDraft>,
        //
    ) -> ProjectAnalysisResult {
        let iter = project_analysis_draft.tree_iter();
        // let iter = a;

        let prompt = project_analysis_draft.prompt.clone();
        // let report = TreeFileWalker::new(move |f| {
        //     println!("File: {}", f.display());

        //     // read entire file
        //     let file_content = read_to_string(f).unwrap();

        //     // format!("{} {}", file_content, prompt);
        //     // self.generate_prompt_for_analysis(&analysis, file_content);
        // })
        // .construct(&mut iter);

        // Vec<(Option<PathBuf>, String)>
        let prompts: Vec<(Option<PathBuf>, String)> = iter
            .map(|event| -> (Option<PathBuf>, String) {
                match event {
                    Ok(event) => match event {
                        Event::File(f) => {
                            let path = f.path().to_path_buf();

                            println!("File: {}", f.path().display());

                            let file_content = read_to_string(f.path()).unwrap_or("".to_string());

                            let max_char = 10_000;

                            let upper = if max_char > file_content.len() {
                                file_content.len()
                            } else {
                                max_char
                            };

                            (
                                Some(path),
                                format!(
                                    "
                                # {}
                                ```
                                {}
                                ```

                                {}:

                                ",
                                    f.path().display(),
                                    file_content.get(..upper).unwrap(),
                                    prompt
                                ), // format!("{} {}", file_content.get(..upper).unwrap(), prompt),
                            )
                        }
                        _ => (None, "".to_string()),
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        (None, "".to_string())
                    }
                }

                // read entire file

                // format!("{} {}", file_content, prompt);
                // self.generate_prompt_for_analysis(&analysis, file_content);
            })
            .filter(|el| if el.0 == None { false } else { true })
            .collect();

        // .collect();

        let mut source_code_analysis: Vec<ProjectSourceFileAnalysis> = Vec::new();

        for (file_name, prompt) in prompts {
            let completion_response = self
                .codex_processor
                .clone()
                .completions_call(prompt.clone(), Some(vec!["#".to_string()]))
                .await;

            let (interpretation, error) = match completion_response.as_ref() {
                Ok(completion) => (completion.choices.first().unwrap().text.trim(), None),
                Err(e) => {
                    println!("Error: {}", e);
                    ("", Some(e.to_string()))
                }
            };

            let file_path = file_name
                .clone()
                .unwrap()
                .as_path()
                .to_string_lossy()
                .to_string();

            println!("{}:\n{}", file_path, interpretation);

            source_code_analysis.push(ProjectSourceFileAnalysis {
                file_path,
                prompt: prompt.clone(),
                analysis: interpretation.to_string(),
                error,
            })

            // let full_script = format!("{}{}", prompt, predicted_commands);

            // let file_path = format!("{}/{}", analysis.project_path.clone(), file_name.unwrap());
            // let file_content = std::fs::read_to_string(file_path).unwrap();

            // analysis.mutations.push(SourceFileMutation::new_from_parent(
            //     SourceFileMutationDraft::new(file_path, prompt),
            //     file_content,
            //     predicted_commands,
            // ));
        }

        ProjectAnalysisResult {
            parent: project_analysis_draft,
            source_files: source_code_analysis,
        }
        // let results = prompts
        //     .iter()
        //     .map(|(file_name, prompt)| async {
        //         let completion = self
        //             .codex_processor
        //             .clone()
        //             .completions_call(prompt.clone(), Some(vec!["#".to_string()]))
        //             .await
        //             .unwrap();

        //         // let predicted_commands = edit.choices.first().unwrap().text.clone();

        //         // let full_script = format!("{}{}", prompt, predicted_commands);

        //         // (file_name.unwrap(), predicted_commands, full_script, prompt)

        //         completion.choices.first().unwrap().text.clone()
        //     })
        //     .collect();

        // println!("Prompts: {:?}", prompts);

        // let file_path = format!("{}/{}", project_path.clone(), file.clone());
        // let file_content = std::fs::read_to_string(file_path.clone()).unwrap();

        // let edit = self
        //     .codex_processor
        //     .clone()
        //     .edit_call(file_content.clone(), "".to_string())
        //     .await
        //     .unwrap();

        // let predicted_commands = edit.choices.first().unwrap().text.clone();

        // let full_script = format!("{}{}", file_content, predicted_commands);

        // ProjectMutation::new_from_parent(
        //     project_path,
        //     file,
        //     file_content,
        //     predicted_commands,
        //     full_script,
        // )
    }
}
