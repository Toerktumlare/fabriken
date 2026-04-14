use std::sync::Arc;

use crate::{
    channels::{GlobalEvent, GlobalSender, StdStreamer},
    executor::{ContainerizeStep, runners::Containerizer},
    models::BuildContext,
    parser::StepId,
};

pub struct ContainerExecutor<'a, C: Containerizer> {
    global_emitter: GlobalSender,
    containerizer: &'a C,
}

impl<'a, C: Containerizer> ContainerExecutor<'a, C> {
    pub fn new(global_emitter: GlobalSender, containerizer: &'a C) -> Self {
        Self {
            global_emitter,
            containerizer,
        }
    }

    pub async fn execute(
        &self,
        ctx: Arc<BuildContext>,
        step: Arc<ContainerizeStep>,
    ) -> anyhow::Result<StepId> {
        let _ = self
            .global_emitter
            .emit(GlobalEvent::ContainerizingStarted)
            .await;
        let log_streamer = StdStreamer::new(self.global_emitter.clone());
        let mut child = self.containerizer.build(&ctx, &step).await?.unwrap();
        log_streamer.process(&step.name, &mut child).await;

        let status = child.wait().await.unwrap();
        if !status.success() {
            anyhow::bail!(
                "Containerizing went wrong! StatusCode: {:?}",
                status.code().unwrap()
            );
        }

        let _ = self
            .global_emitter
            .emit(GlobalEvent::ContainerizingDone)
            .await;

        Ok(step.id)
    }
}
