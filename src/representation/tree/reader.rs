use std::path::Path;

use super::iterator::{Entry, TreeProcessor};

pub type TreeVisitWalkerFunction = fn(&Path);

pub struct TreeFileWalker {
    dir_has_next: Vec<bool>,
    num_dirs: usize,
    num_files: usize,
    visitor: TreeVisitWalkerFunction,
}

impl TreeFileWalker {
    pub fn new(visitor: TreeVisitWalkerFunction) -> Self {
        TreeFileWalker {
            dir_has_next: vec![true],
            num_dirs: 0,
            num_files: 0,
            visitor,
        }
    }
}

impl TreeProcessor for TreeFileWalker {
    fn construct_dir(&mut self, entry: &Entry) -> String {
        self.dir_has_next.pop();
        self.dir_has_next.push(entry.has_next_sibling());

        // Print the relative path to the root dir
        let mut dir = String::new();

        // if self.dir_has_next.is_empty() {
        //     dir.push_str(&self.construct_entry(&entry.path().display()));
        // } else {
        //     dir.push_str(&self.construct_entry(&file_name_from_path(entry.path()).to_string()));
        // };

        self.dir_has_next.push(true);
        self.num_dirs += 1;

        dir
    }

    fn close_dir(&mut self) {
        self.dir_has_next
            .pop()
            .expect("Number of calls to close_dir exceeds open_dir");
    }

    fn construct_file(&mut self, entry: &Entry) -> String {
        self.dir_has_next.pop();
        self.dir_has_next.push(entry.has_next_sibling());

        // let file = self.construct_entry(&file_name_from_path().to_string());
        self.num_files += 1;

        (self.visitor)(entry.path());

        entry.path().display().to_string()
    }
}
