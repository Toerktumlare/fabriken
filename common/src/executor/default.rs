use std::sync::Arc;

use async_trait::async_trait;
use tokio::task::JoinSet;

use crate::{
    channels::{ExecReceiver, ExecutorCommand, SchedSender},
    executor::{ExecuteManager, executors::Executor},
    models::BuildContext,
    parser::GlobalData,
};

pub struct DefaultExecuteManager<E: Executor> {
    executor: Arc<E>,
    to_sched: SchedSender,
    from_sched: ExecReceiver,
    global_data: GlobalData,
}

impl<E> DefaultExecuteManager<E>
where
    E: Executor + Send + Sync,
{
    pub fn new(
        global_data: GlobalData,
        executor: E,
        sender: SchedSender,
        receiver: ExecReceiver,
    ) -> Self {
        Self {
            to_sched: sender,
            from_sched: receiver,
            executor: Arc::new(executor),
            global_data,
        }
    }
}

#[async_trait]
impl<E> ExecuteManager for DefaultExecuteManager<E>
where
    E: Executor + Send + Sync + 'static,
{
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut join_set = JoinSet::new();
        let ctx: Arc<BuildContext> = Arc::new(self.global_data.clone().into());
        while let Some(command) = self.from_sched.recv().await {
            match command {
                ExecutorCommand::RunStep(step) => {
                    let step = step.clone();
                    let executor = self.executor.clone();
                    let to_sched = self.to_sched.clone();
                    let ctx = ctx.clone();
                    join_set.spawn(async move {
                        match executor.execute(ctx, step).await {
                            Ok(id) => to_sched.finished(id).await,
                            Err(_err) => to_sched.failed(0).await,
                        };
                    });
                }
                ExecutorCommand::Shutdown => break,
            }
        }
        Ok(())
    }
}
