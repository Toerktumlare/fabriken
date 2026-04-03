use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncBufRead, AsyncRead};

use crate::build::build_update::UpdateType;
use crate::build::{self, BuildEnd, BuildStart, BuildUpdate, EmptyEvent, LogEntry};
use crate::{executor::ExecutionStep, parser::StepId};

mod exec;
mod global;
mod sched;

pub use exec::{ExecReceiver, ExecSender};
pub use global::{GlobalReceiver, GlobalSender};
pub use sched::{SchedReceiver, SchedSender};

#[derive(Debug)]
pub enum GlobalEvent {
    Log(LogEvent),
    BuildStart,
    BuildEnd,
    PullingImage,
    ImageFetched,
}

impl From<GlobalEvent> for BuildUpdate {
    fn from(value: GlobalEvent) -> Self {
        let update = match value {
            GlobalEvent::Log(log_event) => {
                let stream = match log_event.stream {
                    StreamType::Stdout => build::StreamType::Stdout,
                    StreamType::Stderr => build::StreamType::Stderr,
                };

                UpdateType::Log(LogEntry {
                    step: log_event.step,
                    line: log_event.line,
                    stream_type: stream as i32,
                })
            }
            GlobalEvent::BuildStart => UpdateType::Start(BuildStart {}),
            GlobalEvent::BuildEnd => UpdateType::End(BuildEnd {}),
            GlobalEvent::PullingImage => UpdateType::PullingImage(EmptyEvent {}),
            GlobalEvent::ImageFetched => UpdateType::ImageFetched(EmptyEvent {}),
        };

        BuildUpdate {
            update_type: Some(update),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub step: String,
    pub line: String,
    pub stream: StreamType,
}

#[derive(Debug, Clone)]
pub enum StreamType {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub enum ExecutorCommand {
    RunStep(Arc<ExecutionStep>),
    Shutdown,
}

#[derive(Debug)]
pub enum ExecutorEvent {
    StepFinished(StepId),
    StepFailed(StepId),
}

#[async_trait]
pub trait LogStreamer {
    async fn stream<R>(
        &mut self,
        reader: R,
        name: String,
        stream_type: StreamType,
    ) -> anyhow::Result<()>
    where
        R: AsyncRead + AsyncBufRead + Unpin + Send;
}

#[derive(Debug, Clone)]
pub struct StdStreamer {
    log_sender: GlobalSender,
}

impl StdStreamer {
    pub fn new(log_sender: GlobalSender) -> Self {
        Self { log_sender }
    }
}

#[async_trait]
impl LogStreamer for StdStreamer {
    async fn stream<R>(
        &mut self,
        reader: R,
        name: String,
        stream_type: StreamType,
    ) -> anyhow::Result<()>
    where
        R: AsyncRead + AsyncBufRead + Unpin + Send,
    {
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let log_event = LogEvent {
                step: name.clone(),
                line: line.trim_end().to_string(),
                stream: stream_type.clone(),
            };
            self.log_sender.emit(GlobalEvent::Log(log_event)).await;
        }

        Ok(())
    }
}
