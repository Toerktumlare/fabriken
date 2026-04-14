use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BuildDefinition {
    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub pipeline: Vec<Step>,

    #[serde(skip)]
    pub context: Context,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Step {
    pub name: String,
    pub image: Option<String>,

    #[serde(default)]
    pub run: Vec<String>,

    #[serde(default)]
    pub depends_on: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub containerize: Option<Containerize>,

    #[serde(default)]
    pub push: Option<Push>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Context {
    #[serde(skip)]
    pub project_root: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Containerize {
    pub builder: Builder,
    pub file: PathBuf,
    pub context: PathBuf,
    pub image: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Builder {
    Docker,
    Podman,
    Buildah,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Push {
    artifact: String,
    target: PushTarget,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PushTarget {
    Copy {
        from: PathBuf,
        to: PathBuf,
    },
    Nexus {
        url: String,
        repository: String,
        format: String,
        path: String,

        #[serde(default)]
        auth: Option<Auth>,
    },
    Storage {
        provider: StorageProvider,
        bucket: String,
        key: String,

        #[serde(default)]
        region: Option<String>,

        #[serde(default)]
        auth: Option<Auth>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    S3,
    Minio,
    Azure,
    Gcs,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Auth {
    Basic(BasicAuth),
    Mtls(MtlsAuth),
    Oauth(OAuthAuth),
}

#[derive(Debug, Deserialize, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MtlsAuth {
    pub cert: String,
    pub key: String,
    pub ca: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthAuth {
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,

    #[serde(default)]
    pub scopes: Vec<String>,
}
