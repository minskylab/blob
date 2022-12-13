use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;

use super::tree::{Entry, TreeProcessor};

pub struct TreeRepresentation {
    dir_has_next: Vec<bool>,
    num_dirs: usize,
    num_files: usize,
    // summary_format: SummaryFormat,
}

impl TreeRepresentation {
    pub fn new() -> Self {
        TreeRepresentation {
            dir_has_next: vec![true],
            num_dirs: 0,
            num_files: 0,
        }
    }

    fn construct_entry<D: Display>(&mut self, name: &D) -> String {
        let vertical_line = "│   ";
        let branched_line = "├── ";
        let terminal_line = "└── ";
        let empty_line = "    ";

        let mut entry = String::new();

        let len = self.dir_has_next.len();

        for (i, has_next) in self.dir_has_next.iter().enumerate() {
            if i < len - 1 {
                if *has_next {
                    entry.push_str(vertical_line);
                } else {
                    entry.push_str(empty_line);
                }
            } else if *has_next {
                entry.push_str(branched_line);
            } else {
                entry.push_str(terminal_line);
            }
        }

        entry.push_str(&format!("{}\n", name.to_string()));

        entry
    }
}

fn file_name_from_path(path: &Path) -> Cow<str> {
    path.file_name().unwrap().to_string_lossy()
}

impl TreeProcessor for TreeRepresentation {
    fn construct_dir(&mut self, entry: &Entry) -> String {
        self.dir_has_next.pop();
        self.dir_has_next.push(entry.has_next_sibling());

        // Print the relative path to the root dir
        let mut dir = String::new();

        if self.dir_has_next.is_empty() {
            dir.push_str(&self.construct_entry(&entry.path().display()));
        } else {
            dir.push_str(&self.construct_entry(&file_name_from_path(entry.path()).to_string()));
        };

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

        let file = self.construct_entry(&file_name_from_path(entry.path()).to_string());
        self.num_files += 1;

        file
    }
}
