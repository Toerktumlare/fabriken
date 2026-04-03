use tokio::sync::mpsc;

use crate::{channels::ExecutorEvent, parser::StepId};

#[derive(Debug, Clone)]
pub struct SchedSender {
    tx: mpsc::Sender<ExecutorEvent>,
}

#[derive(Debug)]
pub struct SchedReceiver {
    rx: mpsc::Receiver<ExecutorEvent>,
}

impl SchedSender {
    pub fn new(buffer: usize) -> (Self, SchedReceiver) {
        let (cmd_tx, cmd_rx) = mpsc::channel(buffer);
        (Self { tx: cmd_tx }, SchedReceiver { rx: cmd_rx })
    }
}

impl SchedSender {
    pub async fn finished(&self, step_id: StepId) {
        let _ = self.tx.send(ExecutorEvent::StepFinished(step_id)).await;
    }

    pub async fn failed(&self, step_id: StepId) {
        let _ = self.tx.send(ExecutorEvent::StepFailed(step_id)).await;
    }
}

impl SchedReceiver {
    pub async fn recv(&mut self) -> Option<ExecutorEvent> {
        self.rx.recv().await
    }
}
