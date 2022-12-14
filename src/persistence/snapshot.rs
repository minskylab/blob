use std::process::Command;

pub struct Snapshot {
    path_root: String,

    current_structure: String,
    proposed_structure: String,

    prompt: String,
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
            prompt: prompt,
        }
    }

    pub fn generate_prompt(&self) -> String {
        let pwd_command = Command::new("pwd").output().unwrap();
        let pwd_result = String::from_utf8_lossy(&pwd_command.stdout);

        let ls_command = Command::new("ls").output().unwrap();
        let ls_result = String::from_utf8_lossy(&ls_command.stdout);

        format!(
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
            self.current_structure.replace("\n", "\n# "),
            self.prompt.replace("\n", "\n# "),
            self.path_root.replace("\n", "\n# "),
            self.proposed_structure.replace("\n", "\n# "),
            pwd_result.replace("\n", "\n# "),
            ls_result.replace("\n", "\n# "),
            self.path_root,
        )
    }
}
