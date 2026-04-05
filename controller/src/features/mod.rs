mod agent_info;
mod agent_repository;
mod healthcheck_service;
mod registration_service;

use std::net::SocketAddr;

pub use agent_repository::AgentRepository;
pub use healthcheck_service::HealthcheckService;
pub use registration_service::RegistrationController;

use tonic_health::pb::health_check_response::ServingStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Healthy,
    Unhealthy,
    Disconnected,
    Unknown,
    Offline,
}

#[derive(Debug, Clone)]
pub struct AgentRegistration {
    cpu_cores: u32,
    memory_gb: u32,
    status: AgentStatus,
    addr: SocketAddr,
}

impl From<ServingStatus> for AgentStatus {
    fn from(value: ServingStatus) -> Self {
        match value {
            ServingStatus::Unknown => AgentStatus::Unknown,
            ServingStatus::Serving => AgentStatus::Healthy,
            ServingStatus::NotServing => AgentStatus::Disconnected,
            ServingStatus::ServiceUnknown => AgentStatus::Unhealthy,
        }
    }
}
