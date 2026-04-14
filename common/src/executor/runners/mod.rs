use std::{collections::HashMap, process::Stdio};

use async_trait::async_trait;
use tokio::process::{Child, Command};
use tracing::info;

use crate::{
    executor::{ContainerizeStep, RunStep},
    models::BuildContext,
};

#[derive(Debug, Clone)]
pub enum ContainerRunner {
    Podman(Podman),
    Docker(Docker),
    Buildah(Buildah),
}

#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn run(&self, ctx: &BuildContext, step: &RunStep) -> anyhow::Result<Option<Child>>;
    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>>;
}

#[async_trait]
pub trait Containerizer: Send + Sync {
    async fn build(
        &self,
        ctx: &BuildContext,
        step: &ContainerizeStep,
    ) -> anyhow::Result<Option<Child>>;
}

#[derive(Debug, Clone)]
pub struct Podman;

#[async_trait]
impl CommandRunner for Podman {
    async fn run(&self, ctx: &BuildContext, step: &RunStep) -> anyhow::Result<Option<Child>> {
        info!("running build step: {}", step.name);
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let image = step.image.clone().unwrap();
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
            .arg(image)
            .arg("sh")
            .arg("-c")
            .arg(step.run.join(" && "))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }

    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>> {
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

#[async_trait]
impl Containerizer for Podman {
    async fn build(
        &self,
        ctx: &BuildContext,
        step: &ContainerizeStep,
    ) -> anyhow::Result<Option<Child>> {
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let child = Command::new("podman")
            .arg("build")
            .args(env_args)
            .arg("-f")
            .arg(step.file.as_path().to_string_lossy().to_string())
            .arg("-t")
            .arg(step.image.clone())
            .arg(step.context.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }
}

#[derive(Debug, Clone)]
pub struct Docker;

#[async_trait]
impl CommandRunner for Docker {
    async fn run(&self, ctx: &BuildContext, step: &RunStep) -> anyhow::Result<Option<Child>> {
        info!("running build step: {}", step.name);
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let child = Command::new("docker")
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
            .arg(&step.image.clone().unwrap())
            .arg("sh")
            .arg("-c")
            .arg(step.run.join(" && "))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }

    async fn pull(&self, image: &str) -> anyhow::Result<Option<Child>> {
        let image = image.trim();
        info!("Pulling image {:?}", image);
        let child = Command::new("docker")
            .arg("pull")
            .arg(image)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }
}

#[async_trait]
impl Containerizer for Docker {
    async fn build(
        &self,
        ctx: &BuildContext,
        step: &ContainerizeStep,
    ) -> anyhow::Result<Option<Child>> {
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let child = Command::new("docker")
            .arg("build")
            .args(env_args)
            .arg("-f")
            .arg(step.file.as_path().to_string_lossy().to_string())
            .arg("-t")
            .arg(step.image.clone())
            .arg(step.context.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Some(child))
    }
}

#[derive(Debug, Clone)]
pub struct Buildah;

#[async_trait]
impl Containerizer for Buildah {
    async fn build(
        &self,
        ctx: &BuildContext,
        step: &ContainerizeStep,
    ) -> anyhow::Result<Option<Child>> {
        let envs = merge_envs(&step.env, &ctx.env);
        let env_args = into_args(envs);
        let child = Command::new("buildah")
            .arg("build")
            .args(env_args)
            .arg("-f")
            .arg(step.file.as_path().to_string_lossy().to_string())
            .arg("-t")
            .arg(step.image.clone())
            .arg(step.context.clone())
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
