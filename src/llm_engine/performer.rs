use crate::codex::processor::CodexProcessor;
use crate::representation::{
    tree::{TreeIter, TreeProcessor},
    tree_representation::TreeRepresentation,
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

    pub async fn perform_edit(&mut self, root: &mut TreeIter, prompt: String) -> String {
        let context = self.generate_context(root);

        let edit = self
            .codex_processor
            .clone()
            .codex_edit_call(context, prompt)
            .await
            .unwrap();

        edit.choices.first().unwrap().text.clone()
    }
}
