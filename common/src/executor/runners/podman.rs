use std::{collections::HashMap, process::Stdio};

use async_trait::async_trait;
use tokio::process::{Child, Command};
use tracing::info;

use crate::{
    channels::{GlobalEvent, GlobalSender},
    executor::{ExecutionStep, runners::ContainerRunner},
    models::BuildContext,
};

#[derive(Debug)]
pub struct PodmanRunner {
    event_sender: GlobalSender,
}

impl PodmanRunner {
    pub fn new(sender: GlobalSender) -> Self {
        Self {
            event_sender: sender,
        }
    }
}

#[async_trait]
impl ContainerRunner for PodmanRunner {
    async fn run(&self, ctx: &BuildContext, step: &ExecutionStep) -> anyhow::Result<Option<Child>> {
        info!("running build step: {}", step.name);
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let child = Command::new("podman")
            .arg("run")
            .args(env_args)
            .arg("--rm")
            .arg("-v")
            .arg(format!(
                "{}:/workspace",
                ctx.base_path.as_path().to_string_lossy()
            ))
            .arg("-w")
            .arg("/workspace")
            .arg(&step.image)
            .arg("sh")
            .arg("-c")
            .arg(step.commands.join(" && "))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }

    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>> {
        self.event_sender.emit(GlobalEvent::PullingImage).await;
        let image = image.trim();
        info!("Pulling image {:?}", image);
        let child = Command::new("podman")
            .arg("pull")
            .arg(image)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }
}

// converts a HashMap<String, String> into Vec<String> of format "-e KEY=VALUE"
fn into_args(envs: HashMap<String, String>) -> Vec<String> {
    envs.iter()
        .flat_map(|(k, v)| vec!["-e".into(), format!("{}={}", k, v)])
        .collect()
}

fn merge_envs(
    local: &HashMap<String, String>,
    global: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut envs = global.clone();
    envs.extend(local.clone());
    envs
}
