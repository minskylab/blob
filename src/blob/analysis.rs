use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::representation::tree::{
    filters::{FilterAggregate, GitignoreFilter},
    iterator::TreeIter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisDraft {
    pub path_root: String,
    pub prompt: String,
    // pub structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSourceFileAnalysis {
    pub file_path: String,
    pub prompt: String,
    pub analysis: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisResult {
    pub parent: Box<ProjectAnalysisDraft>,
    pub source_files: Vec<ProjectSourceFileAnalysis>,
}

impl ProjectAnalysisDraft {
    pub fn new(path_root: String, prompt: String) -> Self {
        ProjectAnalysisDraft { path_root, prompt }
    }

    pub fn new_with_default_prompt(path_root: String) -> Self {
        ProjectAnalysisDraft { path_root, prompt: "Please generate a comprehensive, detailed, and specific summary of the following code snippet. Your summary should include the following information:

1. Purpose of the code: what does the code do, what problem does it solve, and what is its intended effect in the context of the overall system or business logic? Provide an overview of the code's main function and any notable behavior.
2. Programming constructs used: what programming language is the code written in, and what specific constructs are used (e.g. functions, classes, loops, conditionals, etc.)? Describe the syntax, purpose, and behavior of the constructs used.
3. Algorithms or data structures employed: does the code use any specific algorithms or data structures (e.g. sorting algorithms, tree structures, etc.)? If so, explain what they are and how they are used in the code. Discuss the time and space complexity of any algorithms used.
4. Business logic inferred from the code: what can you infer about the business logic or system the code is a part of based on the code itself? Provide examples of the inputs, outputs, and processing of the code that support your inference.
5. Notable features or challenges: are there any interesting or challenging aspects of the code that you would like to highlight? This can include efficiency, scalability, maintainability, edge cases, etc.

In your summary, please explicitly state any assumptions or contextual information necessary to understand the code and its behavior within the larger system. Additionally, use appropriate references to any external dependencies, data sources, or other related code snippets as needed.
        ".to_string(),
    }
    }

    fn calculate_tree_iter(&mut self) -> Box<TreeIter> {
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
