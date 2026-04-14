use std::sync::Arc;

use crate::{
    channels::{GlobalEvent, GlobalSender, StdStreamer},
    executor::{
        RunStep,
        runners::{CommandRunner, Podman},
    },
    models::BuildContext,
    parser::StepId,
};

pub struct StepExecutor<R: CommandRunner> {
    runner: R,
    global_emitter: GlobalSender,
}

impl StepExecutor<Podman> {
    pub fn new(global_emitter: GlobalSender) -> Self {
        Self {
            global_emitter,
            runner: Podman,
        }
    }
}

impl<R: CommandRunner> StepExecutor<R> {
    pub async fn execute(
        &self,
        ctx: Arc<BuildContext>,
        step: Arc<RunStep>,
    ) -> anyhow::Result<StepId> {
        let log_streamer = StdStreamer::new(self.global_emitter.clone());

        let _ = self.global_emitter.emit(GlobalEvent::PullingImage).await;
        let mut child = self
            .runner
            .pull(&step.image.clone().unwrap())
            .await?
            .unwrap();
        log_streamer.process(&step.name, &mut child).await;

        let status = child.wait().await.unwrap();
        if !status.success() {
            anyhow::bail!(
                "Pulling image went wrong! StatusCode: {:?}",
                status.code().unwrap()
            );
        }

        let _ = self.global_emitter.emit(GlobalEvent::ImageFetched).await;
        let _ = self.global_emitter.emit(GlobalEvent::StepStarted).await;
        let mut child = self.runner.run(&ctx, &step).await?.unwrap();
        log_streamer.process(&step.name, &mut child).await;

        let status = child.wait().await.unwrap();
        if !status.success() {
            anyhow::bail!(
                "Pulling image went wrong! StatusCode: {:?}",
                status.code().unwrap()
            );
        }

        let _ = self.global_emitter.emit(GlobalEvent::StepEnded).await;
        Ok(step.id)
    }
}
