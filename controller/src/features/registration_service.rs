use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use common::startup::{ConfigResponse, RegistrationRequest, registration_server::Registration};
use tonic::{Request, Response, Status, async_trait};
use tracing::info;

use crate::features::{
    AgentRegistration, HealthcheckService,
    agent_repository::{AgentId, AgentRepository},
};

pub struct RegistrationController {
    pub agent_repository: Arc<AgentRepository>,
    pub healthcheck_service: Arc<HealthcheckService>,
}

#[async_trait]
impl Registration for RegistrationController {
    async fn register(
        &self,
        request: Request<RegistrationRequest>,
    ) -> Result<Response<ConfigResponse>, Status> {
        let body = request.into_inner();
        let agent_id = body.name;
        let addr: SocketAddr = body.addr.parse().unwrap();
        let registration = AgentRegistration {
            cpu_cores: 0,
            memory_gb: 0,
            status: super::AgentStatus::Healthy,
            addr,
        };

        info!("Registering agent: {}", agent_id);
        self.agent_repository
            .register(AgentId(agent_id.clone()), registration)
            .await;

        self.healthcheck_service
            .spawn_healthcheck(AgentId(agent_id.clone()), addr)
            .await
            .unwrap();

        Ok(Response::new(ConfigResponse { success: true }))
    }
}
