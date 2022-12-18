use chrono::serde::ts_seconds_option;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{create_dir_all, File},
    io::Write,
};

// use serde_json::Value;
// use std::time::SystemTime;

// pub struct BlobInteractionResponse {
//     response: String,
//     value: Value,
// }

// enum BlobInteractionKind {
//     Complete,
//     Edit,
//     Insert,
// }

// pub struct BlobInteraction {
//     timestamp: String,
//     prompt: String,

//     kind: BlobInteractionKind,

//     response: Option<Box<BlobInteractionResponse>>,
// }

// pub struct BlobProject {
//     pub root_path: String,
//     dialogue: Vec<Box<BlobInteraction>>,
// }

// impl BlobProject {
//     pub fn new(root_path: String) -> Self {
//         Self {
//             root_path,
//             dialogue: Vec::new(),
//         }
//     }

//     fn calculate_folder_structure(&self) -> String {
//         let file_structure = String::new();

//         file_structure
//     }

//     pub fn interact(&mut self, kind: BlobInteractionKind, prompt: String) {
//         let timestamp = SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .unwrap()
//             .as_secs()
//             .to_string();

//         match kind {
//             BlobInteractionKind::Complete => {
//                 self.dialogue.push(Box::new(BlobInteraction {
//                     timestamp,
//                     prompt,
//                     kind,
//                     response: None,
//                 }));
//             }
//             BlobInteractionKind::Edit => {
//                 self.dialogue.push(Box::new(BlobInteraction {
//                     timestamp,
//                     prompt,
//                     kind,
//                     response: None,
//                 }));
//             }
//             BlobInteractionKind::Insert => {
//                 self.dialogue.push(Box::new(BlobInteraction {
//                     timestamp,
//                     prompt,
//                     kind,
//                     response: None,
//                 }));
//             }
//         }

//         // }

//         // let dialogue = BlobDialogue {
//         //     timestamp,
//         //     interactions: vec![Box::new(interaction)],
//         // };

//         // self.dialogue.push(Box::new(dialogue));
//     }

//     // pub fn get_blob(&self, blob_name: &str) -> Blob {
//     //     let blob_path = self.get_blob_path(blob_name);
//     //     Blob::new(blob_path)
//     // }
// }

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

    pub fn save_new_definition(&self, root_path: String, definition: String) -> BlobDefinition {
        let definitions_root = self.get_definitions_path(root_path.clone());

        fs::create_dir_all(definitions_root.clone()).unwrap();

        let file_path = format!("{}/{}", definitions_root, "user_definitions.md".to_string());
        // let mut file = File::create().unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        let def = BlobDefinition {
            created_at: Some(Utc::now()),
            definition,
        };

        let definition = serde_json::to_string(&def.clone()).unwrap();
        let formatted_definition = format!("{}\n", definition);

        file.write_all(formatted_definition.as_bytes()).unwrap();

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
}
