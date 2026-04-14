use std::path::PathBuf;

use crate::build::{BuildDef, StepDef};
pub mod build_definition;
mod context;

pub use build_definition::BuildDefinition;
pub use build_definition::Context;
pub use build_definition::Step;
pub use context::BuildContext;

impl From<StepDef> for Step {
    fn from(value: StepDef) -> Self {
        Step {
            name: value.name,
            image: value.image,
            run: value.run,
            depends_on: value.depends_on,
            env: value.env,
            containerize: None,
            push: None,
        }
    }
}

impl From<Step> for StepDef {
    fn from(value: Step) -> Self {
        StepDef {
            name: value.name,
            image: value.image,
            run: value.run,
            depends_on: value.depends_on,
            env: value.env,
            containerize: None,
        }
    }
}

impl From<BuildDef> for BuildDefinition {
    fn from(value: BuildDef) -> Self {
        let context = value.context.unwrap();
        BuildDefinition {
            pipeline: value.steps.into_iter().map(|step| step.into()).collect(),
            env: value.env,
            context: Context {
                project_root: PathBuf::from(context.project_root),
            },
        }
    }
}
