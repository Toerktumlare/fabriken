use std::{collections::HashMap, path::PathBuf};

use indexmap::IndexMap;
use serde::Deserialize;

use crate::build::{BuildDef, StepDef};
mod context;

pub use context::BuildContext;

#[derive(Debug, Deserialize)]
pub struct BuildDefinition {
    pub pipeline: IndexMap<String, StepDefinition>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(skip)]
    pub project_root: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StepDefinition {
    pub image: String,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub commands: Vec<String>,

    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl From<StepDefinition> for StepDef {
    fn from(value: StepDefinition) -> Self {
        StepDef {
            image: value.image,
            commands: value.commands,
            depends_on: value.depends_on,
            env: value.env,
        }
    }
}

impl From<StepDef> for StepDefinition {
    fn from(value: StepDef) -> Self {
        StepDefinition {
            image: value.image,
            commands: value.commands,
            depends_on: value.depends_on,
            env: value.env,
        }
    }
}

impl From<BuildDef> for BuildDefinition {
    fn from(value: BuildDef) -> Self {
        BuildDefinition {
            pipeline: value
                .steps
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            project_root: PathBuf::from(value.project_root),
            env: value.env,
        }
    }
}
