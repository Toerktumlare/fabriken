use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    models::BuildContext,
    parser::{ExecutionStep, StepId},
};

mod container;
mod step;

pub use container::ContainerExecutor;
pub use step::StepExecutor;

#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(
        &self,
        ctx: Arc<BuildContext>,
        step: Arc<ExecutionStep>,
    ) -> anyhow::Result<StepId>;
}
