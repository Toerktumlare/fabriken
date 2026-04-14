use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncRead};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::build::build_update::UpdateType;
use crate::build::{self, BuildEnd, BuildStart, BuildUpdate, EmptyEvent, LogEntry};
use crate::executor::ContainerizeStep;
use crate::{executor::RunStep, parser::StepId};

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
    StepStarted,
    StepEnded,
    PullingImage,
    ImageFetched,
    ContainerizingStarted,
    ContainerizingDone,
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
            GlobalEvent::StepEnded => UpdateType::StepEnded(EmptyEvent {}),
            GlobalEvent::StepStarted => UpdateType::StepEnded(EmptyEvent {}),
            GlobalEvent::ContainerizingStarted => UpdateType::ContainerizingStarted(EmptyEvent {}),
            GlobalEvent::ContainerizingDone => UpdateType::ContainerizingDone(EmptyEvent {}),
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
    RunStep(Arc<RunStep>),
    BuildContainer(Arc<ContainerizeStep>),
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
        name: &str,
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

    pub async fn process(&self, name: &str, child: &mut Child) {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_lines = BufReader::new(stdout);
        let stderr_lines = BufReader::new(stderr);

        let out = stream(stdout_lines, name, StreamType::Stdout, &self.log_sender);
        let err = stream(stderr_lines, name, StreamType::Stderr, &self.log_sender);

        let (_, _) = tokio::join!(out, err);
    }
}

async fn stream<R>(
    reader: R,
    name: &str,
    stream_type: StreamType,
    log_sender: &GlobalSender,
) -> anyhow::Result<()>
where
    R: AsyncRead + AsyncBufRead + Unpin + Send,
{
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let log_event = LogEvent {
            step: name.to_string().clone(),
            line: line.trim_end().to_string(),
            stream: stream_type.clone(),
        };
        log_sender.emit(GlobalEvent::Log(log_event)).await;
    }

    Ok(())
}
