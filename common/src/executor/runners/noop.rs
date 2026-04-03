use async_trait::async_trait;
use tokio::process::Child;
use tracing::info;

use crate::{
    channels::{GlobalEvent, GlobalSender, LogEvent, StreamType},
    executor::{ExecutionStep, runners::ContainerRunner},
    models::BuildContext,
};

#[derive(Debug)]
pub struct NoopRunner {
    global: GlobalSender,
}

impl NoopRunner {
    pub fn new(sender: GlobalSender) -> Self {
        Self { global: sender }
    }
}

#[async_trait]
impl ContainerRunner for NoopRunner {
    async fn run(
        &self,
        _ctx: &BuildContext,
        step: &ExecutionStep,
    ) -> anyhow::Result<Option<Child>> {
        self.global
            .emit(GlobalEvent::Log(LogEvent {
                step: step.name.clone(),
                line: "Running build".to_string(),
                stream: StreamType::Stderr,
            }))
            .await;
        info!("Executing step: {}", step.id);
        Ok(None)
    }

    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>> {
        self.global
            .emit(GlobalEvent::Log(LogEvent {
                step: "Pull".to_string(),
                line: "Pulling image".to_string(),
                stream: StreamType::Stderr,
            }))
            .await;
        info!("Pulling image: {}", image);
        Ok(None)
    }
}
