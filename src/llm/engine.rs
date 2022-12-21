use crate::blob::mutation::{
    ProjectMutation, ProjectMutationDraft, ProjectMutationProposed, SourceFileMutation,
    SourceFileMutationDraft,
};
use crate::codex::processor::CodexProcessor;
use crate::representation::{
    tree::iterator::{TreeIter, TreeProcessor},
    tree::representation::TreeRepresentation,
};

pub struct LLMEngine {
    llm_representation: TreeRepresentation,
    codex_processor: CodexProcessor,
}

impl LLMEngine {
    pub fn new() -> Self {
        let access_token = std::env::var("OPENAI_API_KEY").unwrap();

        LLMEngine {
            llm_representation: TreeRepresentation::new(),
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
            .completions_call(next_prompt.clone())
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
}
