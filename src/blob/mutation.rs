use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::{path::Path, process::Command};

use crate::representation::{
    tree::filters::{FilterAggregate, GitignoreFilter},
    tree::iterator::TreeIter,
};

#[derive(Clone, Debug)]
pub struct MutationError(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMutationDraft {
    pub path_root: String,
    pub prompt: String,

    pub context_lines: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    // tree_iter: Option<Box<TreeIter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMutationProposed {
    pub parent: Box<ProjectMutationDraft>,
    current_structure: String,
    proposed_structure: String,
    // state: MutationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMutation {
    pub parent: Box<ProjectMutationProposed>,
    pub predicted_commands: String,
    pub full_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileMutationDraft {
    pub file_path: String,
    pub prompt: String,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileMutation {
    pub parent: Box<SourceFileMutationDraft>,
    pub current_content: String,
    pub proposed_content: String,
}

impl ProjectMutationDraft {
    pub fn new(path_root: String, prompt: String, context_lines: Vec<String>) -> Self {
        ProjectMutationDraft {
            path_root,
            prompt,
            context_lines: Some(context_lines),
            created_at: Utc::now(),
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
}

impl ProjectMutationProposed {
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

# Project Context:
# {}

# Current structure:
# {}
# {}

# User prompt:
# {}

# Expected structure:
# {}
# {}

# Local Context:
# `pwd` 
# {}

# `ls`
# {}

# Unix commands to perform transformation:
cd {}
",
            context_definitions.trim_end().replace('\n', "\n# "),
            self.parent.as_ref().path_root,
            self.current_structure.trim_end().replace('\n', "\n# "),
            self.parent.as_ref().prompt.trim_end().replace('\n', "\n# "),
            self.parent
                .as_ref()
                .path_root
                .trim_end()
                .replace('\n', "\n# "),
            self.proposed_structure.trim_end().replace('\n', "\n# "),
            pwd_result.trim_end().replace('\n', "\n# "),
            ls_result.trim_end().replace('\n', "\n# "),
            self.parent.as_ref().path_root,
        ))
    }
}

impl ProjectMutation {
    pub fn new_from_parent(
        parent: Box<ProjectMutationProposed>,
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

impl SourceFileMutationDraft {
    pub fn new(file_path: String, prompt: String) -> Self {
        SourceFileMutationDraft {
            file_path,
            prompt,
            created_at: Utc::now(),
        }
    }
}

impl SourceFileMutation {
    pub fn new_from_parent(
        parent: Box<SourceFileMutationDraft>,
        current_content: String,
        proposed_content: String,
    ) -> Self {
        Self {
            parent,
            current_content,
            proposed_content,
        }
    }
}
