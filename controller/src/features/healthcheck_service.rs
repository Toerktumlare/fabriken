use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::{
    select,
    sync::{
        RwLock,
        mpsc::{self},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tonic::{Request, transport::Endpoint};
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};
use tracing::{error, info};

use crate::features::{AgentRepository, AgentStatus, agent_repository::AgentId};

type Registry = Arc<RwLock<HashMap<AgentId, CancellationToken>>>;

pub struct HealthcheckService {
    registry: Registry,
    consumer: RwLock<Option<JoinHandle<()>>>,
    reporter: HealthReporter,
    token: CancellationToken,
    agent_repository: Arc<AgentRepository>,
}

impl HealthcheckService {
    pub fn run(agent_repository: Arc<AgentRepository>) -> Self {
        let (tx_health, mut rx_health) = mpsc::channel::<HealthReportEvent>(32);

        let token = CancellationToken::new();
        let master_token = token.clone();
        let agent_repo = agent_repository.clone();
        let registry = Arc::new(RwLock::new(HashMap::<AgentId, CancellationToken>::new()));
        let task_registry = registry.clone();

        let handle = tokio::spawn(async move {
            loop {
                select! {
                    _ = master_token.cancelled() => {
                        break;
                    }
                    Some(event) = rx_health.recv() => {
                        match event {
                            HealthReportEvent::StatusUpdate(id, status) => {
                                info!("Status updated for agent: {:?} to: {:?}", id, status);
                                agent_repo.set_status(&id, status).await;
                            },
                            HealthReportEvent::Cancel(id) => {
                                info!("Cancelling health check for agent: {:?}", id);
                                if let Some(token) = task_registry.write().await.remove(&id){
                                    token.cancel();
                                };

                                info!("Status for agent: {:?} is set to OFFLINE", id);
                                agent_repo.set_status(&id, AgentStatus::Offline).await;
                            },
                            HealthReportEvent::Register(id, token) => {
                                info!("Registering agent: {:?}", id);
                                let mut reg = task_registry.write().await;
                                if let Some(old_token) = reg.insert(id, token) {
                                    old_token.cancel();
                                };
                            },
                        }
                    }
                }
            }
        });

        Self {
            registry,
            consumer: RwLock::new(Some(handle)),
            reporter: HealthReporter { tx: tx_health },
            token,
            agent_repository,
        }
    }

    pub async fn spawn_healthcheck(&self, id: AgentId, addr: SocketAddr) -> Result<()> {
        let token = self.token.child_token();
        let cloned_token = token.clone();
        let reporter = self.reporter.clone();
        let current_id = id.clone();

        tokio::spawn(async move {
            let url = format!("http://{:?}", addr);

            dbg!(&url);

            let Ok(channel) = Endpoint::from_shared(url.clone()).unwrap().connect().await else {
                let _ = reporter.cleanup(current_id).await;
                return;
            };

            let _ = reporter.register(id, token).await;

            let mut health_client = HealthClient::new(channel);
            let mut stream = health_client
                .watch(Request::new(HealthCheckRequest { service: "".into() }))
                .await
                .unwrap()
                .into_inner();

            loop {
                select! {
                    _ = cloned_token.cancelled() => {
                        break;
                    }
                    result = stream.message() => {
                        match result {
                            Ok(Some(message)) => {
                                let status = message.status();
                                let _ = reporter.report(current_id.clone(), status.into()).await;
                            }
                            Ok(None) => {
                                info!("Stream was closed by server at: {}", url);
                                break;
                            }
                            Err(e) => {
                                error!("Did we crash? {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            info!("Cleaning up: {:?}", current_id);
            let _ = reporter.cleanup(current_id).await;
        });

        Ok(())
    }

    pub async fn stop(&self) {
        self.token.cancel();
        let handle = self.consumer.write().await.take();

        if let Some(h) = handle {
            let _ = h.await;
            info!("Healthcheck consumer stopped gracefully.");
        }
    }
}

#[derive(Clone)]
pub struct HealthReporter {
    tx: mpsc::Sender<HealthReportEvent>,
}

impl HealthReporter {
    pub async fn register(&self, id: AgentId, token: CancellationToken) -> anyhow::Result<()> {
        self.tx
            .send(HealthReportEvent::Register(id, token))
            .await
            .map_err(|_| anyhow::anyhow!("Health consumer task dropped"))
    }

    pub async fn report(&self, id: AgentId, status: AgentStatus) -> anyhow::Result<()> {
        self.tx
            .send(HealthReportEvent::StatusUpdate(id, status))
            .await
            .map_err(|_| anyhow::anyhow!("Health consumer task dropped"))
    }

    pub async fn cleanup(&self, id: AgentId) -> anyhow::Result<()> {
        self.tx
            .send(HealthReportEvent::Cancel(id))
            .await
            .map_err(|_| anyhow::anyhow!("Health consumer task dropped"))
    }
}

pub enum HealthReportEvent {
    StatusUpdate(AgentId, AgentStatus),
    Cancel(AgentId),
    Register(AgentId, CancellationToken),
}
