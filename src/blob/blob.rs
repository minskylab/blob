use chrono::serde::ts_seconds_option;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use chrono::{DateTime, Utc};

use crate::transformer::mutation::ProjectMutationScripted;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobDefinition {
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    pub definition: String,
}

pub struct BlobContextProcessor {}

pub enum BlobDefinitionKind {
    User,
    // Meta,
}

impl BlobDefinitionKind {
    fn as_filename(&self) -> &'static str {
        match self {
            BlobDefinitionKind::User => "user.md",
            // BlobDefinitionKind::Meta => "meta.md",
        }
    }
}

impl BlobContextProcessor {
    pub fn new() -> Self {
        Self {}
    }

    fn get_definitions_path(&self, root_path: String) -> String {
        format!("{root_path}/.blob/.definitions")
    }

    fn get_mutations_path(&self, root_path: String) -> String {
        format!("{root_path}/.blob/.mutations")
    }

    pub fn save_new_definition(
        &self,
        root_path: String,
        kind: BlobDefinitionKind,
        definition: String,
    ) -> BlobDefinition {
        let definitions_root = self.get_definitions_path(root_path.clone());

        create_dir_all(definitions_root.clone()).unwrap();

        let file_path = format!("{}/{}", definitions_root, kind.as_filename().to_string());
        // let mut file = File::create().unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        let now = Utc::now();

        let def = BlobDefinition {
            created_at: Some(now),
            definition: definition.clone(),
        };

        let new_line_definition = format!("{}, {}\n", now.to_rfc3339(), definition);

        // let definition = serde_json::to_string(&new_line_definition.clone()).unwrap();
        // let formatted_definition = format!("{}\n", definition);

        file.write_all(new_line_definition.as_bytes()).unwrap();

        def
    }

    pub fn save_new_context(&self, project_scripted_mutation: ProjectMutationScripted) -> String {
        let root = project_scripted_mutation.parent.parent.path_root;

        let timed_name = project_scripted_mutation
            .parent
            .parent
            .created_at
            .format("%Y%m%d%H%M%S")
            .to_string();

        let mutations_path = self.get_mutations_path(root.clone());
        let new_context_path = format!("{mutations_path}/{timed_name}");
        // save project_scripted_mutation.bash_script to file called script.sh
        create_dir_all(new_context_path.clone()).unwrap();

        let bash_script = project_scripted_mutation.full_script;

        let final_script_path = format!("{new_context_path}/script.sh");

        let mut file = File::create(final_script_path.clone()).unwrap();

        let metadata = file.metadata().unwrap();

        let mut permissions = metadata.permissions();

        permissions.set_mode(0o645);

        file.write_all(bash_script.as_bytes()).unwrap();

        final_script_path
    }

    pub fn retrieve_definitions(
        &self,
        root_path: String,
        kind: BlobDefinitionKind,
    ) -> Vec<BlobDefinition> {
        let definitions_root = self.get_definitions_path(root_path.clone());
        let file_path = format!("{}/{}", definitions_root, kind.as_filename().to_string());

        let mut definitions = Vec::new();

        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return definitions,
        };

        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let lines = contents.split("\n");

        for line in lines {
            let parts = line.split(",").collect::<Vec<&str>>();

            if parts.len() == 2 {
                let created_at_str = parts[0].trim();
                let definition = parts[1].trim();

                let created_at = DateTime::parse_from_rfc3339(created_at_str).unwrap();

                let def = BlobDefinition {
                    created_at: Some(DateTime::from_utc(created_at.naive_utc(), Utc)),
                    definition: definition.to_string(),
                };

                definitions.push(def);
            }
        }

        definitions
    }
}
