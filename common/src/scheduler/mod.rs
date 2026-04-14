use async_trait::async_trait;

mod default;

pub use default::DefaultScheduler;

use crate::parser::Pipeline;

#[async_trait]
pub trait Scheduler {
    async fn schedule(&mut self, pipeline: &Pipeline) -> anyhow::Result<()>;
}
