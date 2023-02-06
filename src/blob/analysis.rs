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
