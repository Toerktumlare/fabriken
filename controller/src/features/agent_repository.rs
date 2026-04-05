use std::{collections::HashMap, sync::Arc, time::Instant};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub struct AgentId(pub String);

use tokio::sync::RwLock;

use crate::features::{AgentRegistration, AgentStatus, agent_info::AgentInfo};

pub struct AgentRepository {
    agents: Arc<RwLock<HashMap<AgentId, AgentInfo>>>,
}

impl AgentRepository {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, agent_id: AgentId, info: AgentRegistration) {
        let now = Instant::now();
        let mut map = self.agents.write().await;
        map.insert(
            agent_id,
            AgentInfo {
                cpu_cores: info.cpu_cores,
                memory_gb: info.memory_gb,
                time_registered: now,
                last_update: now,
                status: info.status,
            },
        );
    }

    pub async fn set_status(&self, agent_id: &AgentId, status: AgentStatus) {
        let now = Instant::now();
        let mut map = self.agents.write().await;
        if let Some(info) = map.get_mut(agent_id) {
            info.status = status;
            info.last_update = now;
        };
    }
}
