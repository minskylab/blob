use crate::structure::{growth::DigestedSource, software::Source};

pub enum Task {
    ProcessFile(DigestedSource),
    ProcessDir(DigestedSource),
}

async fn process_task(task: Task) {}
