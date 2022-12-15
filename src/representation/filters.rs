//! Various file filters and abstractions for working with them.

extern crate git2;

use self::git2::Repository;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::result;

type Result = result::Result<bool, Box<dyn Error>>;

/// A filter used to decide whether to include a file in a collection.
pub trait FileFilter {
    /// `Ok(true)` means the file should be included and vice versa.
    fn filter(&self, path: &Path) -> Result;
}

impl<F> FileFilter for F
where
    F: Fn(&Path) -> Result,
{
    fn filter(&self, path: &Path) -> Result {
        (self)(path)
    }
}

/// A collection of filters acting as one.
pub struct FilterAggregate {
    filters: Vec<Box<dyn FileFilter>>,
}

impl FilterAggregate {
    /// Add a filter to the collection.
    pub fn push<F>(&mut self, filter: F)
    where
        F: FileFilter + 'static,
    {
        self.filters.push(Box::new(filter));
    }
}

impl Default for FilterAggregate {
    fn default() -> Self {
        FilterAggregate {
            filters: Vec::new(),
        }
    }
}

impl FileFilter for FilterAggregate {
    fn filter(&self, path: &Path) -> Result {
        for f in &self.filters {
            if !f.filter(path).unwrap() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

// Builder for `GlobFilter`.
// pub struct GlobFilterBuilder {
//     patterns: Vec<String>,
//     invert: bool,
// }

// impl GlobFilterBuilder {
//     /// Create a new builder.
//     ///
//     /// If `invert` is true, matches are inverted.
//     pub fn new(invert: bool) -> Self {
//         GlobFilterBuilder {
//             patterns: Vec::new(),
//             invert: invert,
//         }
//     }

//     /// Add a pattern to the builder.
//     pub fn add(&mut self, pattern: String) -> &mut Self {
//         self.patterns.push(pattern);
//         self
//     }

//     /// Build a `GlobFilter` from the set options.
//     pub fn build(&self) -> result::Result<GlobFilter, Box<dyn Error>> {
//         let mut builder = GlobSetBuilder::new();

//         for pattern in &self.patterns {
//             builder.add(Glob::new(&pattern).unwrap());
//         }

//         builder
//             .build()
//             .map(|set| GlobFilter {
//                 pattern: set,
//                 invert: self.invert,
//             })
//             .map_err(From::from)
//     }
// }

// Filter by glob pattern.
// pub struct GlobFilter {
//     pattern: GlobSet,
//     invert: bool,
// }

// impl GlobFilter {
//     /// Create a new glob filter from an iterator of `String` patterns.
//     ///
//     /// If `invert` is true, matches are inverted.
//     pub fn from<I: Iterator<Item = String>>(
//         patterns: I,
//         invert: bool,
//     ) -> result::Result<GlobFilter, Box<dyn Error>> {
//         let mut builder = GlobFilterBuilder::new(invert);

//         for pattern in patterns {
//             builder.add(pattern);
//         }

//         builder.build()
//     }
// }

// impl FileFilter for GlobFilter {
//     fn filter(&self, path: &Path) -> Result {
//         let path = path.strip_prefix("./").unwrap_or(path);
//         let is_match = self.pattern.is_match(path);

//         Ok(if self.invert { !is_match } else { is_match })
//     }
// }

// /// Exclude files ignored by git.
pub struct GitignoreFilter {
    repo: Repository,
}

impl GitignoreFilter {
    /// Create a new filter rooted at `path`.
    pub fn new(path: PathBuf) -> Option<result::Result<GitignoreFilter, Box<dyn Error>>> {
        let result = Repository::discover(path).map(|repo| GitignoreFilter { repo });

        match result {
            Err(err) => {
                if err.code() == git2::ErrorCode::NotFound {
                    None
                } else {
                    Some(Err(From::from(err)))
                }
            }
            Ok(repo) => Some(Ok(repo)),
        }
    }
}

impl FileFilter for GitignoreFilter {
    fn filter(&self, path: &Path) -> Result {
        // ./filename paths doesn't seem to work with should_ignore
        let path = path.canonicalize();
        self.repo
            .status_should_ignore(&path.unwrap())
            .map(|x| !x)
            .map_err(From::from)
    }
}

// Exclude hidden files.
//
// This function relies on the Unix convention of denoting hidden files with a leading dot (`.`).
// pub fn filter_hidden_files(path: &Path) -> Result {
//     path.file_name()
//         .and_then(|name| name.to_str().map(|str| !str.starts_with('.')))
//         .ok_or_else(|| From::from("No file name."))
// }

// /// Exclude non directory files.
// pub fn filter_non_dirs(path: &Path) -> Result {
//     path.metadata()
//         .map(|data| data.is_dir())
//         .map_err(From::from)
// }
