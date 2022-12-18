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

use crate::transformer::mutation::ProjectMutationScripted;

pub struct BlobContextProcessor {}

impl BlobContextProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn save_new_context(&self, project_scripted_mutation: ProjectMutationScripted) -> String {
        let root = project_scripted_mutation.parent.parent.path_root;

        let timed_name = project_scripted_mutation
            .parent
            .parent
            .created_at
            .format("%Y%m%d%H%M%S")
            .to_string();

        let new_context_path = format!("{root}/.blob/{timed_name}");
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
