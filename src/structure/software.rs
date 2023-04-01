use std::path::PathBuf;

use git2::Repository;
use tokio::fs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source<Payload = ()>
where
    Payload: Clone + Sync,
{
    File {
        path: PathBuf,
        payload: Payload,
    },
    Dir {
        path: PathBuf,
        payload: Payload,
        children: Vec<Source<Payload>>,
    },
}

pub struct Project {
    pub root_path: PathBuf,
    repository: Option<Repository>,
}

impl Project {
    pub fn new(root_path: PathBuf) -> Self {
        let repository = Repository::discover(root_path.clone()).ok();

        Project {
            root_path,
            repository,
        }
    }

    pub async fn calculate_source<Payload, FnBuilder>(
        &mut self,
        mut builder: FnBuilder,
    ) -> Vec<Source<Payload>>
    where
        Payload: Clone + Sync,
        FnBuilder: FnMut(&Source) -> Payload,
    {
        let mut source = vec![];
        let mut to_visit = vec![self.root_path.clone()];

        let with_git_repository = self.repository.is_some();

        while let Some(path) = to_visit.pop() {
            let canon_path = path.clone().canonicalize();

            if with_git_repository
                && self
                    .repository
                    .as_ref()
                    .unwrap()
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

                let files: Vec<Source<()>> = files
                    .into_iter()
                    .map(|file| Source::File {
                        path: file.path(),
                        payload: (),
                    })
                    .collect();

                let d = Source::Dir {
                    path: path.clone(),
                    payload: (),
                    children: files.clone(),
                };

                let new_source = Source::Dir::<Payload> {
                    path: path.clone(),
                    payload: builder(&d),
                    children: files
                        .iter()
                        .map(|atom| match atom {
                            Source::File { path, payload: _ } => Source::File::<Payload> {
                                path: path.clone(),
                                payload: builder(&Source::File {
                                    path: path.clone(),
                                    payload: (),
                                }),
                            },
                            _ => unreachable!(),
                        })
                        .collect::<Vec<Source<Payload>>>(),
                };

                source.push(new_source);
                // source.push(SourceAtom::Dir(path, files));
            } else {
                source.push(Source::File {
                    path: path.clone(),
                    payload: builder(&Source::File {
                        path: path.clone(),
                        payload: (),
                    }),
                });
                // source.push(SourceAtom::File(path));
            }
        }

        source
    }
}
