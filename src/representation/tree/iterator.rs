use std::error::Error;
use std::fmt;
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::representation::tree::filters::FileFilter;

#[derive(Debug)]
pub enum Event {
    File(Entry),
    OpenDir(Entry),
    CloseDir,
}

pub struct Entry {
    path: PathBuf,
    has_next_sibling: bool,
    metadata: fs::Metadata,
}

impl Entry {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn has_next_sibling(&self) -> bool {
        self.has_next_sibling
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter
            .debug_struct("Entry")
            .field("path", &self.path)
            .field("has_next_sibling", &self.has_next_sibling)
            .field("is_dir", &self.metadata.is_dir())
            .finish()
    }
}

struct FilteredDir {
    file_filter: Rc<dyn FileFilter>,
    dir: fs::ReadDir,
}

impl FilteredDir {
    pub fn new<P>(path: P, file_filter: Rc<dyn FileFilter>) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        fs::read_dir(&path)
            .map(|dir| FilteredDir {
                file_filter: file_filter,
                dir: dir,
            })
            .map_err(|err| {
                From::from(format!(
                    "Failed to read dir '{}': {}",
                    path.as_ref().display(),
                    err
                ))
            })
    }
}

impl Iterator for FilteredDir {
    type Item = Result<fs::DirEntry, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = match self.dir.next() {
                Some(result) => result,
                None => return None,
            };

            let entry = match result {
                Ok(entry) => entry,
                Err(err) => return Some(Err(From::from(err))),
            };

            let should_yield = match self.file_filter.filter(entry.path().as_path()) {
                Ok(should_yield) => should_yield,
                Err(err) => return Some(Err(From::from(err))),
            };

            if should_yield {
                return Some(Ok(entry));
            }
        }
    }
}

pub struct TreeIter {
    dir_stack: Vec<Peekable<FilteredDir>>,
    file_filter: Rc<dyn FileFilter>,
}

impl TreeIter {
    /// Create a new iterator with `path` as root.
    pub fn new<F>(path: PathBuf, file_filter: F) -> Result<Self, Box<dyn Error>>
    where
        F: FileFilter + 'static,
    {
        let rc_filter = Rc::new(file_filter);

        let p = path.clone();

        fs::read_dir(p)
            .map(|dir| {
                let filtered = FilteredDir {
                    file_filter: rc_filter.clone(),
                    dir,
                };

                TreeIter {
                    dir_stack: vec![filtered.peekable()],
                    file_filter: rc_filter,
                }
            })
            .map_err(From::from)
    }
}

fn has_next_sibling<T, E, I: Iterator<Item = Result<T, E>>>(dir: &mut Peekable<I>) -> bool {
    loop {
        match dir.peek() {
            Some(result) => {
                if result.is_ok() {
                    return true;
                }
            }
            None => {
                return false;
            }
        }
    }
}

fn next_entry(dir: &mut Peekable<FilteredDir>) -> Option<Result<Entry, Box<dyn Error>>> {
    let entry = match dir.next() {
        Some(Ok(entry)) => entry,
        Some(Err(err)) => return Some(Err(From::from(err))),
        None => return None,
    };

    let has_next_sibling = has_next_sibling(dir);
    let metadata = match entry.metadata() {
        Ok(metadata) => metadata,
        Err(err) => return Some(Err(From::from(err))),
    };
    let path = entry.path();

    Some(Ok(Entry {
        path,
        metadata,
        has_next_sibling,
    }))
}

impl Iterator for TreeIter {
    type Item = Result<Event, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry;

        loop {
            match self.dir_stack.as_mut_slice().last_mut() {
                Some(dir) => match next_entry(dir) {
                    Some(Ok(the_entry)) => {
                        entry = the_entry;
                        break;
                    }
                    Some(Err(err)) => return Some(Err(err)),
                    None => {}
                },
                None => return None,
            };

            self.dir_stack.pop();
            return Some(Ok(Event::CloseDir));
        }

        if entry.metadata.is_dir() {
            match FilteredDir::new(&entry.path, self.file_filter.clone()) {
                Ok(dir) => self.dir_stack.push(dir.peekable()),
                Err(err) => return Some(Err(From::from(err))),
            };

            Some(Ok(Event::OpenDir(entry)))
        } else {
            Some(Ok(Event::File(entry)))
        }
    }
}

pub trait TreeProcessor {
    fn close_dir(&mut self);

    fn construct_dir(&mut self, entry: &Entry) -> String;
    fn construct_file(&mut self, entry: &Entry) -> String;

    fn construct(&mut self, tree: &mut TreeIter) -> Result<String, Box<dyn Error>> {
        let mut result = String::new();
        for event in tree {
            match event {
                Ok(event) => {
                    match event {
                        Event::OpenDir(ref entry) => result.push_str(&self.construct_dir(entry)),
                        Event::File(ref entry) => result.push_str(&self.construct_file(entry)),
                        Event::CloseDir => self.close_dir(),
                    };
                }
                Err(err) => return Err(err),
            };
        }

        Ok(result)
    }
}
