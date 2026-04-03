use std::sync::Arc;

use async_trait::async_trait;

use crate::{executor::ExecutionStep, models::BuildContext, parser::StepId};

mod default;

pub use default::DefaultExecutor;

#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(
        &self,
        ctx: Arc<BuildContext>,
        step: Arc<ExecutionStep>,
    ) -> anyhow::Result<StepId>;
}
