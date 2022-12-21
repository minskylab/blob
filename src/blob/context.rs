use chrono::serde::ts_seconds;
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

use crate::blob::mutation::ProjectMutation;
use crate::blob::mutation::SourceFileMutation;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct BlobDefinition {
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlobMutationMetadataKind {
    Project(Box<ProjectMutation>),
    SourceFile(Box<SourceFileMutation>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct BlobMutationMetadata {
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub kind: BlobMutationKind,
    pub mutation: BlobMutationMetadataKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlobMutationKind {
    Project,
    SourceFile,
}

pub struct BlobContextProcessor {
    project_path: String,
}

pub enum BlobDefinitionKind {
    Project,
    SelfReference,
    // Meta,
}
impl BlobDefinitionKind {
    fn as_filename(&self) -> &'static str {
        match self {
            BlobDefinitionKind::Project => "_project.md",
            BlobDefinitionKind::SelfReference => "_self.md",
            // BlobDefinitionKind::Meta => "meta.md",
        }
    }
}

impl BlobContextProcessor {
    pub fn new(project_path: String) -> Self {
        let ctx = Self { project_path };

        ctx.save_definitions(BlobDefinitionKind::SelfReference, false, vec![
            "blob is a cli tool to modify the source code with natural language instructions".to_string(),
            "blob uses OpenAI GPT-3 to understand the instructions and execute the mutations in form of unix instructions or file editing".to_string(),
            "blob have two commands: `blob do` an `blob define`".to_string(),
            "`blob do \"YOUR_INSTRUCTION\"` is used to execute a mutation over the entire project file structure".to_string(),
            "`blob do -f <file> \"YOUR_INSTRUCTION\"` is used to execute a mutation over specific file".to_string(),
            "`blob define \"YOUR_DEFINITION\"` is used to define a concept and improve the context for blob".to_string(),
        ]);

        ctx
    }

    fn get_definitions_path(&self) -> String {
        format!("{}/.blob/.definitions", self.project_path)
    }

    fn get_mutations_path(&self) -> String {
        format!("{}/.blob/.mutations", self.project_path)
    }

    fn get_project_mutation_path(&self, project_mutation: ProjectMutation) -> String {
        let context_name = project_mutation
            .parent
            .parent
            .created_at
            .format("%Y%m%d%H%M%S")
            .to_string();

        let mutations_path = self.get_mutations_path();

        format!("{mutations_path}/{context_name}")
    }

    fn get_source_file_mutation_path(&self, source_file_mutation: SourceFileMutation) -> String {
        let context_name = source_file_mutation
            .parent
            .created_at
            .format("%Y%m%d%H%M%S")
            .to_string();

        let mutations_path = self.get_mutations_path();

        format!("{mutations_path}/{context_name}")
    }

    pub fn save_project_mutation(&self, project_mutation: ProjectMutation) -> String {
        let new_context_path = self.get_project_mutation_path(project_mutation.clone());

        let final_script_path = format!("{new_context_path}/script.sh");
        let metadata_path = format!("{new_context_path}/metadata.json");

        create_dir_all(new_context_path.clone()).unwrap();

        let bash_script = project_mutation.clone().full_script;

        let mut script_file = File::create(final_script_path.clone()).unwrap();

        let metadata = script_file.metadata().unwrap();

        let mut permissions = metadata.permissions();

        permissions.set_mode(0o645);

        script_file.write_all(bash_script.as_bytes()).unwrap();

        let mut metadata_file = File::create(metadata_path.clone()).unwrap();

        let metadata = BlobMutationMetadata {
            created_at: project_mutation.parent.parent.created_at,
            kind: BlobMutationKind::Project,
            mutation: BlobMutationMetadataKind::Project(Box::new(project_mutation)),
        };

        let metadata_json = serde_json::to_string(&metadata).unwrap();

        metadata_file.write_all(metadata_json.as_bytes()).unwrap();

        final_script_path
    }

    pub fn save_source_file_mutation(&self, source_file_mutation: SourceFileMutation) -> String {
        let new_context_path = self.get_source_file_mutation_path(source_file_mutation.clone());

        // source_file_mutation.clone().parent.file_path
        let mutated_source_file_path = format!(
            "{}/{}",
            new_context_path,
            source_file_mutation.clone().parent.file_path
        );

        let directories_only = source_file_mutation
            .clone()
            .parent
            .file_path
            .clone()
            .split("/")
            .collect::<Vec<&str>>()
            .split_last()
            .unwrap()
            .1
            .join("/");

        let directories_mutated_source_file_path = format!("{new_context_path}/{directories_only}");

        create_dir_all(directories_mutated_source_file_path.clone()).unwrap();

        // let mutated_source_file_path = format!("{mutated_script_path}/script.sh");
        let metadata_path = format!("{new_context_path}/metadata.json");

        let mut file_source_file = File::create(mutated_source_file_path.clone()).unwrap();

        let source_file_content = source_file_mutation.clone().proposed_content;

        file_source_file
            .write_all(source_file_content.as_bytes())
            .unwrap();

        let mut metadata_file = File::create(metadata_path.clone()).unwrap();

        let metadata = BlobMutationMetadata {
            created_at: source_file_mutation.parent.created_at,
            kind: BlobMutationKind::SourceFile,
            mutation: BlobMutationMetadataKind::SourceFile(Box::new(source_file_mutation)),
        };

        let metadata_json = serde_json::to_string(&metadata).unwrap();

        metadata_file.write_all(metadata_json.as_bytes()).unwrap();
        // create_dir_all(directories_onlydd.clone()).unwrap();

        // let mutations_path = self.get_mutations_path(self.project_path.clone());
        // let new_context_path = format!("{mutations_path}/{context_name}");

        // source_file_mutation.parent.file_path.clone()
        // self.get_mutations_path()
        mutated_source_file_path.to_string()
    }

    // pub fn save_project_definition(
    //     &self,
    //     kind: BlobDefinitionKind,
    //     definition: String,
    // ) -> BlobDefinition {
    //     let definitions_root = self.get_definitions_path();

    //     create_dir_all(definitions_root.clone()).unwrap();

    //     let file_path = format!("{}/{}", definitions_root, kind.as_filename().to_string());
    //     // let mut file = File::create().unwrap();

    //     let mut file = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .append(true)
    //         .open(file_path)
    //         .unwrap();

    //     let now = Utc::now();

    //     let def = BlobDefinition {
    //         created_at: now,
    //         definition: definition.clone(),
    //     };

    //     let new_line_definition = format!("{}, {}\n", now.to_rfc3339(), definition);

    //     // let definition = serde_json::to_string(&new_line_definition.clone()).unwrap();
    //     // let formatted_definition = format!("{}\n", definition);

    //     file.write_all(new_line_definition.as_bytes()).unwrap();

    //     def
    // }

    pub fn retrieve_definitions(&self, kind: BlobDefinitionKind) -> Vec<BlobDefinition> {
        let definitions_root = self.get_definitions_path();
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
                    created_at: DateTime::from_utc(created_at.naive_utc(), Utc),
                    definition: definition.to_string(),
                };

                definitions.push(def);
            }
        }

        definitions
    }

    fn save_definitions(
        &self,
        kind: BlobDefinitionKind,
        append: bool,
        definitions: Vec<String>,
    ) -> Vec<BlobDefinition> {
        let definitions_root = self.get_definitions_path();

        create_dir_all(definitions_root.clone()).unwrap();

        let file_path = format!("{}/{}", definitions_root, kind.as_filename().to_string());
        // let mut file = File::create().unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .open(file_path)
            .unwrap();

        let now = Utc::now();

        let mut final_definitions: Vec<BlobDefinition> = Vec::new();

        for definition in definitions {
            let def = BlobDefinition {
                created_at: now,
                definition: definition.clone(),
            };

            let new_line_definition =
                format!("{}, {}\n", def.created_at.to_rfc3339(), def.definition);

            file.write_all(new_line_definition.as_bytes()).unwrap();

            final_definitions.push(def);
        }

        final_definitions
    }

    pub fn save_project_definitions(&self, definitions: Vec<String>) -> Vec<BlobDefinition> {
        self.save_definitions(BlobDefinitionKind::Project, true, definitions)
    }
}
