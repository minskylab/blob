use std::process::Command;

pub enum SnapshotState {
    Created,
    Proposed,
}

#[derive(Debug)]
pub struct SnapshotError(String);

pub struct Snapshot {
    path_root: String,

    prompt: String,

    current_structure: String,
    proposed_structure: String,

    state: SnapshotState,
}

impl Snapshot {
    pub fn new(
        path_root: String,
        current_structure: String,
        proposed_structure: String,
        prompt: String,
    ) -> Self {
        Self {
            path_root,
            current_structure,
            proposed_structure,
            prompt,
            state: SnapshotState::Proposed,
        }
    }

    pub fn generate_prompt(&self) -> Result<String, SnapshotError> {
        match self.state {
            SnapshotState::Created => Err(SnapshotError(
                "Cannot generate prompt from a early created snapshot".to_string(),
            )),
            SnapshotState::Proposed => {
                let pwd_command = Command::new("pwd").output().unwrap();
                let pwd_result = String::from_utf8_lossy(&pwd_command.stdout);

                let ls_command = Command::new("ls").output().unwrap();
                let ls_result = String::from_utf8_lossy(&ls_command.stdout);

                Ok(format!(
                    "
#!/bin/bash
# Context

# current structure:
# {}
# {}

# prompt:
# {}

# proposed structure:
# {}
# {}

# `pwd` 
# {}

# `ls`
# {}

# unix commands to perform this transformation:
cd {}
",
                    self.path_root,
                    self.current_structure.trim_end().replace("\n", "\n# "),
                    self.prompt.trim_end().replace("\n", "\n# "),
                    self.path_root.trim_end().replace("\n", "\n# "),
                    self.proposed_structure.trim_end().replace("\n", "\n# "),
                    pwd_result.trim_end().replace("\n", "\n# "),
                    ls_result.trim_end().replace("\n", "\n# "),
                    self.path_root,
                ))
            }
        }
    }
}
