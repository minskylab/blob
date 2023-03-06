//! Various file filters and abstractions for working with them.

extern crate git2;

use self::git2::Repository;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::result;

type Result = result::Result<bool, Box<dyn Error>>;

pub trait FileFilter {
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
        let path = path.canonicalize();
        self.repo
            .status_should_ignore(&path.unwrap())
            .map(|x| !x)
            .map_err(From::from)
    }
}
