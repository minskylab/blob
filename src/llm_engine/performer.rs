use crate::codex::processor::CodexProcessor;
use crate::representation::{
    tree::{TreeIter, TreeProcessor},
    tree_representation::TreeRepresentation,
};
use crate::transformer::mutation::{
    ProjectMutationDraft, ProjectMutationExtended, ProjectMutationScripted,
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

    pub async fn generate_proposal(
        &mut self,
        mut mutation_draft: Box<ProjectMutationDraft>,
        // prompt: String,
    ) -> Box<ProjectMutationExtended> {
        let mut root_tree = mutation_draft.tree_iter();
        let context = self.generate_context(root_tree.as_mut());

        let prompt = mutation_draft.prompt();

        let edit = self
            .codex_processor
            .clone()
            .edit_call(context.clone(), prompt.clone())
            .await
            .unwrap();

        Box::new(ProjectMutationExtended::new_from_parent(
            mutation_draft,
            context.clone(),
            edit.choices.first().unwrap().text.clone(),
        ))
    }

    pub async fn generate_bash_script(
        &mut self,
        mutation_draft: Box<ProjectMutationDraft>,
        // prompt: String,
    ) -> ProjectMutationScripted {
        // let prompt = mutation_init.prompt();

        let snapshot = self.generate_proposal(mutation_draft).await;

        let next_prompt = snapshot.clone().generate_prompt().unwrap();

        let completion = self
            .codex_processor
            .clone()
            .completions_call(next_prompt.clone())
            .await
            .unwrap();

        let full_script = format!(
            "{}{}",
            next_prompt,
            completion.choices.first().unwrap().text
        );

        ProjectMutationScripted::new_from_parent(snapshot.clone(), full_script)

        // completion.choices.first().unwrap().text.clone()
    }
}
