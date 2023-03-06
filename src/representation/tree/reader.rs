use futures::{stream, Stream, StreamExt};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, DirEntry},
    io,
}; // 0.3.1

use super::iterator::{Entry, TreeProcessor};

pub type TreeVisitWalkerFunction = fn(path: &Path, is_dir: bool);

pub struct TreeFileWalker {
    dir_has_next: Vec<bool>,
    num_dirs: usize,
    num_files: usize,
    visitor: TreeVisitWalkerFunction,
}

pub async fn visit_file_path(
    path: impl Into<PathBuf>,
) -> impl Stream<Item = io::Result<DirEntry>> + Send + 'static {
    async fn one_level(path: PathBuf, to_visit: &mut Vec<PathBuf>) -> io::Result<Vec<DirEntry>> {
        let mut dir = fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(child) = dir.next_entry().await? {
            if child.metadata().await?.is_dir() {
                to_visit.push(child.path());
            } else {
                files.push(child)
            }
        }

        Ok(files)
    }

    stream::unfold(vec![path.into()], |mut to_visit| async {
        let path = to_visit.pop()?;
        let file_stream = match one_level(path, &mut to_visit).await {
            Ok(files) => stream::iter(files).map(Ok).left_stream(),
            Err(e) => stream::once(async { Err(e) }).right_stream(),
        };

        Some((file_stream, to_visit))
    })
    .flatten()
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

        // if self.dir_has_next.is_empty() {
        //     dir.push_str(&self.construct_entry(&entry.path().display()));
        // } else {
        //     dir.push_str(&self.construct_entry(&file_name_from_path(entry.path()).to_string()));
        // };

        self.dir_has_next.push(true);
        self.num_dirs += 1;

        (self.visitor)(entry.path(), true);

        entry.path().display().to_string()
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

        (self.visitor)(entry.path(), false);

        entry.path().display().to_string()
    }
}
