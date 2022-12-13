//! Types for recursively walking the file system tree.

use super::filters::FileFilter;
use std::error::Error;
use std::fmt;
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Events yielded from `TreeIter`.
#[derive(Debug)]
pub enum Event {
    /// Any non-directory file in the current directory.
    File(Entry),
    /// A directory contained within the current directory. This is the new current directory.
    OpenDir(Entry),
    /// Signals end of current directory. The parent becomes the new current directory.
    CloseDir,
}

/// Represents an entry in the file system.
pub struct Entry {
    path: PathBuf,
    has_next_sibling: bool,
    metadata: fs::Metadata,
}

impl Entry {
    /// Path to the entry, relative to its root.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Whether the iterator that yielded this entry has more sibling (same directory) entries.
    pub fn has_next_sibling(&self) -> bool {
        self.has_next_sibling
    }

    /// A cached metadata entry for this file. It's probably better to use this than
    /// calling `fs::metadata` on `path`.
    pub fn metadata(&self) -> &fs::Metadata {
        &self.metadata
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

/// An iterator yielding only the entries in dir where `file_filter` returns true.
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

/// A filtered recursive directory iterator.
///
/// The iterator descends the tree depth first. This means that all of a directory's children
/// will immediately follow their parent. This essentially mirrors the output of this program.
///
/// # Example
/// Given the following directory structure, where directories are denoted by a trailing slash,
/// the items would be returned from `TreeIter` in the same order.
///
/// ```text
/// .
/// ├── a
/// ├── b/
/// │   ├── 1
/// │   └── 2
/// ├── c/
/// └── d
/// ```
///
/// This would be the yielded events, in order:
///
/// ```text
/// File(a)
/// OpenDir(b)
/// File(1)
/// File(2)
/// CloseDir
/// OpenDir(c)
/// CloseDir
/// File(d)
/// ```
pub struct TreeIter {
    dir_stack: Vec<Peekable<FilteredDir>>,
    file_filter: Rc<dyn FileFilter>,
}

impl TreeIter {
    /// Create a new iterator with `path` as root.
    pub fn new<P, F>(path: P, file_filter: F) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
        F: FileFilter + 'static,
    {
        let rc_filter = Rc::new(file_filter);

        fs::read_dir(path)
            .map(|dir| {
                let filtered = FilteredDir {
                    file_filter: rc_filter.clone(),
                    dir: dir,
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
        path: path,
        metadata: metadata,
        has_next_sibling: has_next_sibling,
    }))
}

impl Iterator for TreeIter {
    type Item = Result<Event, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry;

        loop {
            match self.dir_stack.as_mut_slice().last_mut() {
                Some(dir) => {
                    match next_entry(dir) {
                        Some(Ok(the_entry)) => {
                            entry = the_entry;
                            break;
                        }
                        Some(Err(err)) => return Some(Err(err)),
                        // Top dir is empty, go down a level by falling through
                        None => {}
                    }
                }
                // We reached top of dir stack
                None => return None,
            };

            // Pop here to avoid multiple mutable references
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

/// A generic trait for processing the output of `TreeIter`.
pub trait TreeProcessor {
    /// Called for each `OpenDir` event.
    // fn open_dir(&mut self, entry: &Entry);
    /// Called for each `CloseDir` event.
    fn close_dir(&mut self);
    /// Called for each `File` event.
    // fn file(&mut self, entry: &Entry);

    fn construct_dir(&mut self, entry: &Entry) -> String;
    fn construct_file(&mut self, entry: &Entry) -> String;
    /// Iterates thorugh a `TreeIter`, delegating each event to its respective method.
    // fn process(&mut self, tree: &mut TreeIter) -> Option<Box<dyn Error>> {
    //     for result in tree {
    //         match result {
    //             Ok(event) => {
    //                 match event {
    //                     Event::OpenDir(ref entry) => self.open_dir(entry),
    //                     Event::File(ref entry) => self.file(entry),
    //                     Event::CloseDir => self.close_dir(),
    //                 };
    //             }
    //             Err(err) => return Some(err),
    //         };
    //     }

    //     None
    // }

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
