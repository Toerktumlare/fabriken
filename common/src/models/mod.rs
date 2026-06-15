use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;

use crate::build::{BuildDef, ContainerizeDef, StepDef};
use crate::models::build_definition::{Builder, Containerize};
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
            containerize: value.containerize.map(Into::into),
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
            containerize: value.containerize.map(Into::into),
        }
    }
}
impl From<ContainerizeDef> for Containerize {
    fn from(value: ContainerizeDef) -> Self {
        Self {
            builder: to_build_enum(value.builder),
            file: PathBuf::from(value.file),
            context: PathBuf::from(value.context),
            image: value.image,
        }
    }
}

impl From<Containerize> for ContainerizeDef {
    fn from(value: Containerize) -> Self {
        Self {
            builder: value.builder.to_string(),
            file: value.file.to_string_lossy().to_string(),
            context: value.context.to_string_lossy().to_string(),
            image: value.image,
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

impl Display for Builder {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let s = match self {
            Builder::Podman => "PODMAN",
            Builder::Docker => "DOCKER",
            Builder::Buildah => "BUILDAH",
        };
        write!(f, "{}", s)
    }
}

fn to_build_enum(value: String) -> Builder {
    match value.as_str() {
        "PODMAN" => Builder::Podman,
        "DOCKER" => Builder::Docker,
        "BUILDAH" => Builder::Buildah,
        _ => todo!(),
    }
}
