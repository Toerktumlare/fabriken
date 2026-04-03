use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::BufReader;

use crate::{
    channels::{GlobalEvent, GlobalSender, LogStreamer, StdStreamer, StreamType},
    executor::{ExecutionStep, executors::Executor, runners::ContainerRunner},
    models::BuildContext,
    parser::StepId,
};

pub struct DefaultExecutor<R: ContainerRunner> {
    runner: R,
    global_emitter: GlobalSender,
}

impl<R: ContainerRunner> DefaultExecutor<R> {
    pub fn new(runner: R, global_emitter: GlobalSender) -> Self {
        Self {
            runner,
            global_emitter,
        }
    }
}

#[async_trait]
impl<R: ContainerRunner> Executor for DefaultExecutor<R> {
    async fn execute(
        &self,
        ctx: Arc<BuildContext>,
        step: Arc<ExecutionStep>,
    ) -> anyhow::Result<StepId> {
        let log_streamer = StdStreamer::new(self.global_emitter.clone());

        let child = self.runner.pull(&step.image).await.unwrap();

        let Some(mut child) = child else {
            return Ok(step.id);
        };

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_lines = BufReader::new(stdout);
        let stderr_lines = BufReader::new(stderr);

        let mut stdout_streamer = log_streamer.clone();
        let mut stderr_streamer = log_streamer.clone();
        let (_, _) = tokio::join!(
            stdout_streamer.stream(stdout_lines, step.name.clone(), StreamType::Stdout),
            stderr_streamer.stream(stderr_lines, step.name.clone(), StreamType::Stderr),
        );

        let status = child.wait().await.unwrap();
        if !status.success() {
            anyhow::bail!(
                "Pulling image went wrong! StatusCode: {:?}",
                status.code().unwrap()
            );
        }

        let _ = self.global_emitter.emit(GlobalEvent::ImageFetched).await;

        let child = self.runner.run(&ctx, &step).await.unwrap();

        let mut child = child.unwrap();

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_lines = BufReader::new(stdout);
        let stderr_lines = BufReader::new(stderr);

        let mut stdout_streamer = log_streamer.clone();
        let mut stderr_streamer = log_streamer.clone();
        let (_, _) = tokio::join!(
            stdout_streamer.stream(stdout_lines, step.name.clone(), StreamType::Stdout),
            stderr_streamer.stream(stderr_lines, step.name.clone(), StreamType::Stderr),
        );

        Ok(step.id)
    }
}
