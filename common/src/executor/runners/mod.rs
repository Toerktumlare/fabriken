use async_trait::async_trait;
use tokio::process::Child;

use crate::{executor::ExecutionStep, models::BuildContext};

mod noop;
mod podman;

pub use noop::NoopRunner;
pub use podman::PodmanRunner;

#[async_trait]
pub trait ContainerRunner: Send + Sync {
    async fn run(&self, ctx: &BuildContext, step: &ExecutionStep) -> anyhow::Result<Option<Child>>;
    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>>;
}
