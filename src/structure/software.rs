use std::path::PathBuf;

use git2::Repository;
use tokio::fs;

// #[derive(Clone, Debug)]
// pub enum SourceAtom {
//     File(PathBuf),
//     Dir(PathBuf, Vec<SourceAtom>),
// }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceAtomTyped<T = ()>
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
            repository,
            // source: vec![],
        }
    }

    pub async fn calculate_source<T>(
        &mut self,
        builder: fn(&SourceAtomTyped) -> T,
    ) -> Vec<SourceAtomTyped<T>>
    where
        T: Clone,
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

                let files: Vec<SourceAtomTyped> = files
                    .into_iter()
                    .map(|file| {
                        // let f = SourceAtom::File(file.path());
                        SourceAtomTyped::File(file.path(), ())
                    })
                    .collect();

                let d = SourceAtomTyped::Dir(path.clone(), files.clone(), ());
                source.push(SourceAtomTyped::Dir(
                    path.clone(),
                    files
                        .iter()
                        .map(|atom| match atom {
                            SourceAtomTyped::File(path, ()) => SourceAtomTyped::File(
                                path.clone(),
                                builder(&SourceAtomTyped::File(path.clone(), ())),
                            ),
                            SourceAtomTyped::Dir(_, _, _) => SourceAtomTyped::Dir(
                                path.clone(),
                                vec![],
                                builder(&SourceAtomTyped::Dir(path.clone(), vec![], ())),
                            ),
                        })
                        .collect(),
                    builder(&d),
                ));
                // source.push(SourceAtom::Dir(path, files));
            } else {
                let f = SourceAtomTyped::File(path.clone(), ());
                source.push(SourceAtomTyped::File(path.clone(), builder(&f)));
                // source.push(SourceAtom::File(path));
            }
        }

        source
    }
}
