use std::sync::Arc;

use tokio::sync::mpsc;

use crate::{
    channels::ExecutorCommand,
    executor::{ContainerizeStep, RunStep},
};

#[derive(Debug)]
pub struct ExecSender {
    tx: mpsc::Sender<ExecutorCommand>,
}

#[derive(Debug)]
pub struct ExecReceiver {
    rx: mpsc::Receiver<ExecutorCommand>,
}

impl ExecSender {
    pub fn new(buffer: usize) -> (Self, ExecReceiver) {
        let (cmd_tx, cmd_rx) = mpsc::channel(buffer);
        (Self { tx: cmd_tx }, ExecReceiver { rx: cmd_rx })
    }
}

impl ExecSender {
    pub async fn run_step(&self, step: Arc<RunStep>) {
        let _ = self.tx.send(ExecutorCommand::RunStep(step.clone())).await;
    }

    pub async fn run_containerize_step(&self, step: Arc<ContainerizeStep>) {
        let _ = self
            .tx
            .send(ExecutorCommand::BuildContainer(step.clone()))
            .await;
    }

    pub async fn shutdown(&self) {
        let _ = self.tx.send(ExecutorCommand::Shutdown).await;
    }
}

impl ExecReceiver {
    pub async fn recv(&mut self) -> Option<ExecutorCommand> {
        self.rx.recv().await
    }
}
