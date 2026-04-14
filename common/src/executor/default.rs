use std::sync::Arc;

use async_trait::async_trait;
use tokio::task::JoinSet;
use tracing::error;

use crate::{
    channels::{ExecReceiver, ExecutorCommand, GlobalSender, SchedSender},
    executor::{
        ExecuteManager,
        executors::{ContainerExecutor, StepExecutor},
        runners::ContainerRunner,
    },
    models::BuildContext,
    parser::GlobalData,
};

pub struct DefaultExecuteManager {
    global_sender: GlobalSender,
    to_sched: SchedSender,
    from_sched: ExecReceiver,
    global_data: GlobalData,
}

impl DefaultExecuteManager {
    pub fn new(
        global_sender: GlobalSender,
        global_data: GlobalData,
        sender: SchedSender,
        receiver: ExecReceiver,
    ) -> Self {
        Self {
            global_sender,
            to_sched: sender,
            from_sched: receiver,
            global_data,
        }
    }
}

#[async_trait]
impl ExecuteManager for DefaultExecuteManager {
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut join_set = JoinSet::new();
        let ctx: Arc<BuildContext> = Arc::new(self.global_data.clone().into());
        while let Some(command) = self.from_sched.recv().await {
            match command {
                ExecutorCommand::RunStep(step) => {
                    let to_sched = self.to_sched.clone();
                    let ctx = ctx.clone();
                    let executor = StepExecutor::new(self.global_sender.clone());
                    join_set.spawn(async move {
                        match executor.execute(ctx, step).await {
                            Ok(id) => to_sched.finished(id).await,
                            // todo! add id to error
                            Err(err) => {
                                error!("Build failed {}", err);
                                to_sched.failed(0).await;
                            }
                        };
                    });
                }
                ExecutorCommand::BuildContainer(step) => {
                    let step = step.clone();
                    let to_sched = self.to_sched.clone();
                    let ctx = ctx.clone();
                    let global_sender = self.global_sender.clone();
                    join_set.spawn(async move {
                        match step.executor_engine {
                            ContainerRunner::Podman(ref podman) => {
                                let executor = ContainerExecutor::new(global_sender, podman);
                                match executor.execute(ctx, step.clone()).await {
                                    Ok(id) => to_sched.finished(id).await,
                                    // todo! add id to error
                                    Err(_err) => to_sched.failed(0).await,
                                };
                            }
                            ContainerRunner::Docker(ref docker) => {
                                let executor = ContainerExecutor::new(global_sender, docker);
                                match executor.execute(ctx, step.clone()).await {
                                    Ok(id) => to_sched.finished(id).await,
                                    // todo! add id to error
                                    Err(_err) => to_sched.failed(0).await,
                                };
                            }
                            ContainerRunner::Buildah(ref buildah) => {
                                let executor = ContainerExecutor::new(global_sender, buildah);
                                match executor.execute(ctx, step.clone()).await {
                                    Ok(id) => to_sched.finished(id).await,
                                    // todo! add id to error
                                    Err(_err) => to_sched.failed(0).await,
                                };
                            }
                        }
                    });
                }
                ExecutorCommand::Shutdown => break,
            }
        }
        Ok(())
    }
}
