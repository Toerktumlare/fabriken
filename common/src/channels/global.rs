use tokio::sync::mpsc;

use crate::channels::GlobalEvent;

#[derive(Debug, Clone)]
pub struct GlobalSender {
    tx: mpsc::Sender<GlobalEvent>,
}

#[derive(Debug)]
pub struct GlobalReceiver {
    rx: mpsc::Receiver<GlobalEvent>,
}

impl GlobalSender {
    pub fn new(buffer: usize) -> (GlobalSender, GlobalReceiver) {
        let (tx, rx) = mpsc::channel(buffer);
        (Self { tx }, GlobalReceiver { rx })
    }
}

impl GlobalSender {
    pub async fn emit(&self, event: GlobalEvent) {
        let _ = self.tx.send(event).await;
    }
}

impl GlobalReceiver {
    pub async fn recv(&mut self) -> Option<GlobalEvent> {
        self.rx.recv().await
    }
}
