use std::{path::Path, process::Command, time::SystemTime};

use crate::{
    llm_engine::performer::LLMEngine,
    representation::{
        filters::{FilterAggregate, GitignoreFilter},
        tree::TreeIter,
    },
};

#[derive(Copy, Clone, Debug)]
pub enum MutationState {
    Created,
    Proposed,
    Extended,
}

#[derive(Clone, Debug)]
pub struct MutationError(String);

// #[derive(Clone, Debug)]
pub struct MutationExtended {
    // parent: Box<&'a mut MutationInit<'a>>,
    parent: Box<MutationInit>,
    current_structure: String,
    proposed_structure: String,
    // state: MutationState,
}

// #[derive(Debug)]
pub struct MutationInit {
    path_root: String,
    prompt: String,

    created_at: SystemTime,
    tree_iter: Option<Box<TreeIter>>,
}

impl MutationInit {
    pub fn new(path_root: String, prompt: String) -> Self {
        let mut s = MutationInit {
            path_root,
            prompt,
            created_at: SystemTime::now(),
            tree_iter: None,
            // tree_iter: Box::new(tree_iter),
        };

        s.tree_iter = Some(s.calculate_tree_iter());

        s
    }

    fn calculate_tree_iter(&self) -> Box<TreeIter> {
        let mut filters = FilterAggregate::default();

        let root = Path::new(&self.path_root).to_owned();

        let github_filter = GitignoreFilter::new(root.clone()).unwrap().unwrap();

        filters.push(github_filter);

        Box::new(TreeIter::new(root, filters).unwrap())
    }

    pub fn tree_iter(&mut self) -> Box<TreeIter> {
        self.calculate_tree_iter()
    }
}

impl MutationExtended {
    // pub fn new_full(
    //     path_root: String,
    //     current_structure: String,
    //     proposed_structure: String,
    //     prompt: String,
    // ) -> Self {
    //     Self {
    //         path_root,
    //         current_structure,
    //         proposed_structure,
    //         prompt,
    //         state: MutationState::Proposed,
    //     }
    // }

    // pub fn new_from_root(path_root: String) -> Self {
    //     Self {
    //         path_root,
    //         current_structure: String::new(),
    //         proposed_structure: String::new(),
    //         prompt: String::new(),
    //         state: MutationState::Created,
    //     }
    // }
    pub fn new_from_parent(
        parent: Box<MutationInit>,
        current_structure: String,
        proposed_structure: String,
    ) -> Self {
        Self {
            parent,
            current_structure,
            proposed_structure,
            // state: MutationState::Created,
        }
    }

    // pub async fn generate_proposal(&mut self, engine: &mut LLMEngine, prompt: String) {
    //     let mut filters = FilterAggregate::default();

    //     let root = Path::new(&self.path_root).to_owned();

    //     let github_filter = GitignoreFilter::new(root.clone()).unwrap().unwrap();

    //     filters.push(github_filter);

    //     let mut tree_iter = TreeIter::new(root, filters).unwrap();
    //     let snp = engine
    //         .generate_proposal(tree_iter.by_ref(), prompt.clone())
    //         .await;

    //     *self = snp;

    //     // self
    // }

    // pub async fn extend_with_llm(&mut self, engine: &mut LLMEngine) {
    //     let mut filters = FilterAggregate::default();

    //     let root = Path::new(&self.path_root).to_owned();

    //     let github_filter = GitignoreFilter::new(root.clone()).unwrap().unwrap();

    //     filters.push(github_filter);

    //     let mut tree_iter = TreeIter::new(root, filters).unwrap();

    //     let completed_bash = engine
    //         .generate_transformer(tree_iter.by_ref(), self.prompt.clone())
    //         .await;

    //     self.prompt = completed_bash;
    // }

    pub fn generate_prompt(self) -> Result<String, MutationError> {
        let pwd_command = Command::new("pwd").output().unwrap();
        let pwd_result = String::from_utf8_lossy(&pwd_command.stdout);

        let ls_command = Command::new("ls").output().unwrap();
        let ls_result = String::from_utf8_lossy(&ls_command.stdout);

        Ok(format!(
            "
#!/bin/bash
# Context

# current structure:
# {}
# {}

# prompt:
# {}

# proposed structure:
# {}
# {}

# `pwd` 
# {}

# `ls`
# {}

# unix commands to perform this transformation:
cd {}
",
            self.parent.as_ref().path_root,
            self.current_structure.trim_end().replace("\n", "\n# "),
            self.parent.as_ref().prompt.trim_end().replace("\n", "\n# "),
            self.parent
                .as_ref()
                .path_root
                .trim_end()
                .replace("\n", "\n# "),
            self.proposed_structure.trim_end().replace("\n", "\n# "),
            pwd_result.trim_end().replace("\n", "\n# "),
            ls_result.trim_end().replace("\n", "\n# "),
            self.parent.as_ref().path_root,
        ))
    }
}