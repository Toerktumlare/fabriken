use std::{collections::HashMap, path::PathBuf};

use async_trait::async_trait;

use crate::{
    executor::runners::{Buildah, ContainerRunner, Docker, Podman},
    models::{Step, build_definition::Builder},
    parser::StepId,
};

mod default;
pub mod executors;
pub mod runners;

pub use default::DefaultExecuteManager;

#[async_trait]
pub trait ExecuteManager {
    async fn run(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct RunStep {
    pub name: String,
    pub run: Vec<String>,
    pub image: Option<String>,
    pub id: StepId,
    pub env: HashMap<String, String>,
}

impl From<(StepId, Step)> for RunStep {
    fn from(value: (StepId, Step)) -> Self {
        let (id, value) = value;
        Self {
            name: value.name,
            image: value.image,
            run: value.run,
            id,
            env: value.env,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerizeStep {
    pub name: String,
    pub id: StepId,
    pub env: HashMap<String, String>,
    pub file: PathBuf,
    pub context: PathBuf,
    pub image: String,
    pub executor_engine: ContainerRunner,
}

impl From<(StepId, Step)> for ContainerizeStep {
    fn from(value: (StepId, Step)) -> Self {
        let (id, step) = value;
        let containerize = step.containerize.unwrap();
        Self {
            name: step.name,
            id,
            env: step.env,
            file: containerize.file,
            context: containerize.context,
            image: containerize.image,
            executor_engine: containerize.builder.into(),
        }
    }
}

impl From<Builder> for ContainerRunner {
    fn from(value: Builder) -> Self {
        match value {
            Builder::Podman => ContainerRunner::Podman(Podman),
            Builder::Docker => ContainerRunner::Docker(Docker),
            Builder::Buildah => ContainerRunner::Buildah(Buildah),
        }
    }
}
