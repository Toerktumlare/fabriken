use std::collections::HashMap;

use async_trait::async_trait;

use crate::{models::StepDefinition, parser::StepId};

mod default;
pub mod executors;
pub mod runners;

pub use default::DefaultExecuteManager;

#[async_trait]
pub trait ExecuteManager {
    async fn run(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub name: String,
    pub commands: Vec<String>,
    pub image: String,
    pub id: StepId,
    pub env: HashMap<String, String>,
}

impl From<(StepId, StepDefinition, String)> for ExecutionStep {
    fn from(value: (StepId, StepDefinition, String)) -> Self {
        let (id, value, name) = value;
        Self {
            name,
            image: value.image,
            commands: value.commands,
            id,
            env: value.env,
        }
    }
}
