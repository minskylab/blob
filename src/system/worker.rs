use crate::structure::{growth::DigestedSource, software::Source};

pub enum Task {
    ProcessDigestedSource(DigestedSource),
}

async fn _process_task(task: Task) {
    // match task {
    //     Task::ProcessDigestedSource(digested) => digested.into(),
    // }
}
