use std::time::Instant;

use crate::features::AgentStatus;

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub last_update: Instant,
    pub status: AgentStatus,
    pub time_registered: Instant,
}
