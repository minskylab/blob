use std::path::Path;

use crate::codex::processor::CodexProcessor;
use crate::representation::{
    tree::{TreeIter, TreeProcessor},
    tree_representation::TreeRepresentation,
};
use crate::transformer::mutation::Mutation;

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
        root_tree: &mut TreeIter,
        prompt: String,
    ) -> Mutation {
        let context = self.generate_context(root_tree);

        let edit = self
            .codex_processor
            .clone()
            .edit_call(context.clone(), prompt.clone())
            .await
            .unwrap();

        Mutation::new_full(
            root_tree.root().to_string_lossy().to_string(),
            context.clone(),
            edit.choices.first().unwrap().text.clone(),
            prompt.clone(),
        )
    }

    pub async fn generate_transformer(&mut self, root: &mut TreeIter, prompt: String) -> String {
        let snapshot = self.generate_proposal(root, prompt).await;

        let next_prompt = snapshot.generate_prompt().unwrap();

        let completion = self
            .codex_processor
            .clone()
            .completions_call(next_prompt.clone())
            .await
            .unwrap();

        completion.choices.first().unwrap().text.clone()
    }
}
