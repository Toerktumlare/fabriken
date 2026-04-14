use crate::{
    executor::{ContainerizeStep, RunStep},
    models::{BuildContext, BuildDefinition, Step},
};
use async_trait::async_trait;
use petgraph::{algo::is_cyclic_directed, prelude::DiGraphMap};
use std::{collections::HashMap, path::PathBuf};

mod pipeline;

pub use pipeline::DefaultPipelineProducer;

#[async_trait]
pub trait PipelineProducer: Send + Sync + 'static {
    async fn produce(definition: BuildDefinition) -> anyhow::Result<(GlobalData, Pipeline)>;
}

pub type StepId = usize;

#[derive(Debug, Default)]
pub struct Pipeline {
    pub graph: DiGraphMap<StepId, String>,
    steps: HashMap<StepId, ExecutionStep>,
    ids: HashMap<String, StepId>,
    next_id: StepId,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
            steps: HashMap::new(),
            ids: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_step(&mut self, step_def: Step) -> StepId {
        let id = self.next_id;
        let name = step_def.name.clone();

        if step_def.containerize.is_some() {
            let step: ContainerizeStep = (id, step_def).into();
            self.steps.insert(id, ExecutionStep::ContainerizeStep(step));
        } else {
            let step: RunStep = (id, step_def).into();
            self.steps.insert(id, ExecutionStep::RunStep(step));
        }

        self.graph.add_node(id);
        self.ids.insert(name, id);
        self.next_id += 1;

        id
    }

    pub fn get_step(&self, step_id: &StepId) -> Option<ExecutionStep> {
        self.steps.get(step_id).cloned()
    }

    pub fn depends_on(&mut self, step: &String, dependency: &String) {
        let step = self.ids.get(step).unwrap();
        let dependency = self.ids.get(dependency).unwrap();
        self.graph.add_edge(*dependency, *step, "".to_owned());
    }

    pub fn is_cyclic(&mut self) -> bool {
        is_cyclic_directed(&self.graph)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalData {
    pub project_root: PathBuf,
    pub env: HashMap<String, String>,
}

impl From<BuildContext> for GlobalData {
    fn from(value: BuildContext) -> Self {
        Self {
            project_root: value.base_path,
            env: value.env,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionStep {
    RunStep(RunStep),
    ContainerizeStep(ContainerizeStep),
}
