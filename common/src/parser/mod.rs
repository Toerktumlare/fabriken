use crate::{
    executor::ExecutionStep,
    models::{BuildContext, BuildDefinition, StepDefinition},
};
use async_trait::async_trait;
use petgraph::{algo::is_cyclic_directed, prelude::DiGraphMap};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

mod pipeline;

pub use pipeline::DefaultPipelineProducer;

#[async_trait]
pub trait PipelineProducer: Send + Sync + 'static {
    async fn produce(definition: BuildDefinition) -> anyhow::Result<Pipeline>;
}

pub type StepId = usize;

#[derive(Debug, Default)]
pub struct Pipeline {
    pub graph: DiGraphMap<StepId, String>,
    steps: HashMap<StepId, Arc<ExecutionStep>>,
    ids: HashMap<String, StepId>,
    next_id: StepId,
    pub project_root: PathBuf,
    pub env: HashMap<String, String>,
}

impl Pipeline {
    pub fn new(project_root: PathBuf, env: HashMap<String, String>) -> Self {
        Self {
            graph: DiGraphMap::new(),
            steps: HashMap::new(),
            ids: HashMap::new(),
            next_id: 0,
            project_root,
            env,
        }
    }

    pub fn add_step(&mut self, name: &str, step_def: StepDefinition) -> StepId {
        let id = self.next_id;
        let step = (id, step_def, name.to_owned()).into();
        self.graph.add_node(id);
        self.steps.insert(id, Arc::new(step));
        self.ids.insert(name.to_owned(), id);
        self.next_id += 1;
        id
    }

    pub fn depends_on(&mut self, step: &String, dependency: &String) {
        let step = self.ids.get(step).unwrap();
        let dependency = self.ids.get(dependency).unwrap();
        self.graph.add_edge(*dependency, *step, "".to_owned());
    }

    pub fn is_cyclic(&mut self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    pub fn split(self) -> (GlobalData, SchedulerData) {
        let Pipeline {
            graph,
            steps,
            ids,
            next_id: _,
            project_root,
            env,
        } = self;

        (
            GlobalData { project_root, env },
            SchedulerData { graph, steps, ids },
        )
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

pub struct SchedulerData {
    pub graph: DiGraphMap<StepId, String>,
    steps: HashMap<StepId, Arc<ExecutionStep>>,
    ids: HashMap<String, StepId>,
}

impl SchedulerData {
    pub fn get_step(&self, step_id: &StepId) -> Option<Arc<ExecutionStep>> {
        self.steps.get(step_id).cloned()
    }
}
