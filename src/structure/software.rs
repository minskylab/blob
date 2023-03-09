use std::path::PathBuf;

use git2::Repository;
use tokio::fs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceAtom<T = ()>
where
    T: Clone,
{
    File(PathBuf, T),
    Dir(PathBuf, Vec<SourceAtom<T>>, T),
}

pub struct Project {
    pub root_path: PathBuf,
    repository: Repository,
}

impl Project {
    pub fn new(root_path: PathBuf) -> Self {
        let repository = Repository::discover(root_path.clone()).unwrap();

        Project {
            root_path,
            repository,
        }
    }

    pub async fn calculate_source<T, B>(&mut self, mut builder: B) -> Vec<SourceAtom<T>>
    where
        T: Clone,
        B: FnMut(&SourceAtom) -> T,
    {
        let mut source = vec![];
        let mut to_visit = vec![self.root_path.clone()];

        while let Some(path) = to_visit.pop() {
            let canon_path = path.clone().canonicalize();
            if self
                .repository
                .status_should_ignore(&canon_path.unwrap())
                .unwrap()
            {
                continue;
            }

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
                        SourceAtom::File(file.path(), ())
                    })
                    .collect();

                let d = SourceAtom::Dir(path.clone(), files.clone(), ());
                source.push(SourceAtom::Dir(
                    path.clone(),
                    files
                        .iter()
                        .map(|atom| match atom {
                            SourceAtom::File(path, ()) => SourceAtom::File(
                                path.clone(),
                                builder(&SourceAtom::File(path.clone(), ())),
                            ),
                            SourceAtom::Dir(_, _, _) => SourceAtom::Dir(
                                path.clone(),
                                vec![],
                                builder(&SourceAtom::Dir(path.clone(), vec![], ())),
                            ),
                        })
                        .collect(),
                    builder(&d),
                ));
                // source.push(SourceAtom::Dir(path, files));
            } else {
                let f = SourceAtom::File(path.clone(), ());
                source.push(SourceAtom::File(path.clone(), builder(&f)));
                // source.push(SourceAtom::File(path));
            }
        }

        source
    }
}
