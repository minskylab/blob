use crate::codex::processor::CodexProcessor;
use crate::representation::{
    tree::{TreeIter, TreeProcessor},
    tree_representation::TreeRepresentation,
};
use crate::transformer::mutation::{MutationInit, MutationProposal};

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
        mut mutation_init: Box<MutationInit>,
        prompt: String,
    ) -> MutationProposal {
        let mut root_tree = mutation_init.tree_iter();
        let context = self.generate_context(root_tree.as_mut());

        let edit = self
            .codex_processor
            .clone()
            .edit_call(context.clone(), prompt.clone())
            .await
            .unwrap();

        MutationProposal::new_from_parent(
            mutation_init,
            context.clone(),
            edit.choices.first().unwrap().text.clone(),
        )
    }

    pub async fn generate_transformer(
        &mut self,
        mutation_init: Box<MutationInit>,
        prompt: String,
    ) -> String {
        let snapshot = self.generate_proposal(mutation_init, prompt).await;

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
