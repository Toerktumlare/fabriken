use async_trait::async_trait;

use crate::{
    channels::{ExecSender, GlobalReceiver, GlobalSender, SchedSender},
    executor::{DefaultExecuteManager, ExecuteManager},
    parser::{GlobalData, Pipeline},
    runtime::Runtime,
    scheduler::{DefaultScheduler, Scheduler},
};

pub struct DefaultRuntime;

#[async_trait]
impl Runtime for DefaultRuntime {
    async fn run(pipeline: Pipeline, global_data: GlobalData) -> GlobalReceiver {
        let (exec_tx, exec_rx) = ExecSender::new(50);
        let (sched_tx, sched_rx) = SchedSender::new(50);
        let (global_tx, global_rx) = GlobalSender::new(50);

        // spawn scheduler
        let sched_global = global_tx.clone();
        tokio::spawn(async move {
            let mut scheduler = DefaultScheduler::new(exec_tx, sched_rx, sched_global);
            scheduler.schedule(&pipeline).await.unwrap();
        });

        // spawn executor
        let exec_global = global_tx.clone();
        tokio::spawn(async move {
            let mut manager =
                DefaultExecuteManager::new(exec_global, global_data, sched_tx, exec_rx);
            manager.run().await.unwrap();
        });

        global_rx
    }
}
