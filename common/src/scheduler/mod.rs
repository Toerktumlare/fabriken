use async_trait::async_trait;

use crate::parser::SchedulerData;

mod default;

pub use default::DefaultScheduler;

#[async_trait]
pub trait Scheduler {
    async fn schedule(&mut self, pipeline: &SchedulerData) -> anyhow::Result<()>;
}
