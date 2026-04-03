use std::collections::HashMap;

use async_trait::async_trait;
use petgraph::{
    Direction::{Incoming, Outgoing},
    visit::EdgeRef,
};
use tracing::info;

use crate::{
    channels::{ExecSender, ExecutorEvent, GlobalEvent, GlobalSender, SchedReceiver},
    parser::SchedulerData,
    scheduler::Scheduler,
};

#[derive(Debug)]
pub struct DefaultScheduler {
    wait_counts: HashMap<usize, usize>,
    to_exec: ExecSender,
    from_exec: SchedReceiver,
    global_tx: GlobalSender,
}

impl DefaultScheduler {
    pub fn new(exec_tx: ExecSender, sched_tx: SchedReceiver, global_tx: GlobalSender) -> Self {
        Self {
            wait_counts: HashMap::new(),
            to_exec: exec_tx,
            from_exec: sched_tx,
            global_tx,
        }
    }
}

#[async_trait]
impl Scheduler for DefaultScheduler {
    async fn schedule(&mut self, pipeline: &SchedulerData) -> anyhow::Result<()> {
        let graph = &pipeline.graph;

        self.global_tx.emit(GlobalEvent::BuildStart).await;

        // add all node counts
        for node in graph.nodes() {
            let count = graph.edges_directed(node, Incoming).count();
            self.wait_counts.insert(node, count);

            // execute all steps that have 0 dependencies
            if count == 0 {
                let step = pipeline.get_step(&node).unwrap();
                self.to_exec.run_step(step.clone()).await;
            }
        }

        // when step has been built, update counts and queue upp all nodes that have 0 dependencies
        loop {
            if let ExecutorEvent::StepFinished(id) = self.from_exec.recv().await.unwrap() {
                info!("Step finished: {:?}", id);
                self.wait_counts.remove(&id);

                if self.wait_counts.is_empty() {
                    let _ = self.to_exec.shutdown().await;
                    break;
                }

                info!("Looking up dependencies for node: {:?}", id);
                for edge in graph.edges_directed(id, Outgoing) {
                    let next = edge.target();
                    info!("Next node to build is: {next}");
                    if let Some(count) = self.wait_counts.get_mut(&next) {
                        *count -= 1;
                        if *count == 0 {
                            let step = pipeline.get_step(&next).unwrap();
                            self.to_exec.run_step(step.clone()).await;
                        }
                    }
                }
            }
        }
        // All steps have been executed we are done!
        self.global_tx.emit(GlobalEvent::BuildEnd).await;

        Ok(())
    }
}
