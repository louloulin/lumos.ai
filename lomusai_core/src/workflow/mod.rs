//! Workflow module for executing sequences of steps

pub struct Workflow {
    pub id: String,
    pub name: String,
}

impl Workflow {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
} 