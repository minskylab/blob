use serde_json::Value;
use std::time::SystemTime;

pub struct BlobInteractionResponse {
    response: String,
    value: Value,
}

enum BlobInteractionKind {
    Complete,
    Edit,
    Insert,
}

pub struct BlobInteraction {
    timestamp: String,
    prompt: String,

    kind: BlobInteractionKind,

    response: Option<Box<BlobInteractionResponse>>,
}

pub struct BlobProject {
    pub root_path: String,
    dialogue: Vec<Box<BlobInteraction>>,
}

impl BlobProject {
    pub fn new(root_path: String) -> Self {
        Self {
            root_path,
            dialogue: Vec::new(),
        }
    }

    fn calculate_folder_structure(&self) -> String {
        let file_structure = String::new();

        file_structure
    }

    pub fn interact(&mut self, kind: BlobInteractionKind, prompt: String) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        match kind {
            BlobInteractionKind::Complete => {
                self.dialogue.push(Box::new(BlobInteraction {
                    timestamp,
                    prompt,
                    kind,
                    response: None,
                }));
            }
            BlobInteractionKind::Edit => {
                self.dialogue.push(Box::new(BlobInteraction {
                    timestamp,
                    prompt,
                    kind,
                    response: None,
                }));
            }
            BlobInteractionKind::Insert => {
                self.dialogue.push(Box::new(BlobInteraction {
                    timestamp,
                    prompt,
                    kind,
                    response: None,
                }));
            }
        }

        // }

        // let dialogue = BlobDialogue {
        //     timestamp,
        //     interactions: vec![Box::new(interaction)],
        // };

        // self.dialogue.push(Box::new(dialogue));
    }

    // pub fn get_blob(&self, blob_name: &str) -> Blob {
    //     let blob_path = self.get_blob_path(blob_name);
    //     Blob::new(blob_path)
    // }
}
