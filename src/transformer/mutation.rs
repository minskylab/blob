use chrono::{DateTime, Utc};
use std::{path::Path, process::Command};

use crate::representation::{
    filters::{FilterAggregate, GitignoreFilter},
    tree::iterator::TreeIter,
};

#[derive(Clone, Debug)]
pub struct MutationError(String);

#[derive(Clone, Debug)]

pub struct ProjectMutationDraft {
    pub path_root: String,
    pub prompt: String,

    pub context_lines: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    // tree_iter: Option<Box<TreeIter>>,
}

#[derive(Clone, Debug)]

pub struct ProjectMutationExtended {
    pub parent: Box<ProjectMutationDraft>,
    current_structure: String,
    proposed_structure: String,
    // state: MutationState,
}

#[derive(Clone, Debug)]
pub struct ProjectMutationScripted {
    pub parent: Box<ProjectMutationExtended>,
    pub predicted_commands: String,
    pub full_script: String,
}

// #[derive(Debug)]

impl ProjectMutationDraft {
    pub fn new(path_root: String, prompt: String, context_lines: Vec<String>) -> Self {
        ProjectMutationDraft {
            path_root,
            prompt,
            context_lines: Some(context_lines),
            created_at: Utc::now(),
            // tree_iter: None,
        }
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

    pub fn prompt(&self) -> &str {
        &self.prompt
    }
}

impl ProjectMutationExtended {
    pub fn new_from_parent(
        parent: Box<ProjectMutationDraft>,
        current_structure: String,
        proposed_structure: String,
    ) -> Self {
        Self {
            parent,
            current_structure,
            proposed_structure,
        }
    }

    pub fn generate_prompt(self) -> Result<String, MutationError> {
        let pwd_command = Command::new("pwd").output().unwrap();
        let pwd_result = String::from_utf8_lossy(&pwd_command.stdout);

        let ls_command = Command::new("ls").output().unwrap();
        let ls_result = String::from_utf8_lossy(&ls_command.stdout);

        let context_definitions = self.parent.clone().context_lines.unwrap().join("\n");

        Ok(format!(
            "#!/bin/bash

# Context:
# {}

# Current structure:
# {}
# {}

# Desire prompt:
# {}

# Proposed structure:
# {}
# {}

# `pwd` 
# {}

# `ls`
# {}

# Unix commands to perform this transformation:
cd {}
",
            context_definitions.trim_end().replace("\n", "\n# "),
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

impl ProjectMutationScripted {
    pub fn new_from_parent(
        parent: Box<ProjectMutationExtended>,
        predicted_commands: String,
        full_script: String,
    ) -> Self {
        Self {
            parent,
            predicted_commands,
            full_script,
        }
    }
}
