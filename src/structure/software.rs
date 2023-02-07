use std::path::PathBuf;

use git2::Repository;
use tokio::fs;

#[derive(Clone, Debug)]
pub enum SourceAtom {
    File(PathBuf),
    Dir(PathBuf, Vec<SourceAtom>),
}

#[derive(Clone, Debug)]
pub enum SourceAtomTyped<T>
where
    T: Clone,
{
    File(PathBuf, T),
    Dir(PathBuf, Vec<SourceAtomTyped<T>>, T),
}

pub struct Project {
    pub root_path: PathBuf,
    repository: Repository,
    // source: Vec<SourceAtom<String>>,
}

impl Project {
    pub fn new(root_path: PathBuf) -> Self {
        let repository = Repository::discover(root_path.clone()).unwrap();

        Project {
            root_path,
            repository: repository,
            // source: vec![],
        }
    }

    pub async fn calculate_source<T>(
        &mut self,
        builder: fn(SourceAtom) -> T,
    ) -> Vec<SourceAtomTyped<T>>
    where
        T: Clone,
    {
        let mut source = vec![];
        // let mut dir_has_next = vec![true];
        // let mut num_dirs = 0;
        // let mut num_files = 0;

        let mut to_visit = vec![self.root_path.clone()];

        while let Some(path) = to_visit.pop() {
            let canon_path = path.clone().canonicalize();
            if self
                .repository
                // .as_ref()
                // .unwrap()
                .status_should_ignore(&canon_path.unwrap())
                // .map(|x| !x)
                // .map_err(|e| e.to_string())
                .unwrap()
            // .unwrap_or(true)
            {
                continue;
            }

            // println!("Visiting: {}", path.display());

            if path.is_dir() {
                let mut dir = fs::read_dir(path.clone()).await.unwrap();
                let mut files = vec![];

                while let Some(child) = dir.next_entry().await.unwrap() {
                    if child.metadata().await.unwrap().is_dir() {
                        to_visit.push(child.path());
                    } else {
                        files.push(child)
                    }
                }

                let files: Vec<SourceAtom> = files
                    .into_iter()
                    .map(|file| {
                        // let f = SourceAtom::File(file.path());
                        SourceAtom::File(file.path())
                    })
                    .collect();

                let d = SourceAtom::Dir(path.clone(), files.clone());
                source.push(SourceAtomTyped::Dir(
                    path.clone(),
                    files
                        .iter()
                        .map(|atom| match atom {
                            SourceAtom::File(path) => SourceAtomTyped::File(
                                path.clone(),
                                builder(SourceAtom::File(path.clone())),
                            ),
                            SourceAtom::Dir(_, _) => SourceAtomTyped::Dir(
                                path.clone(),
                                vec![],
                                builder(SourceAtom::Dir(path.clone(), vec![])),
                            ),
                        })
                        .collect(),
                    builder(d),
                ));
                // source.push(SourceAtom::Dir(path, files));
            } else {
                let f = SourceAtom::File(path.clone());
                source.push(SourceAtomTyped::File(path.clone(), builder(f)));
                // source.push(SourceAtom::File(path));
            }
        }

        source
    }
}
