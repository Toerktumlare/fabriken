use std::{collections::HashMap, path::PathBuf};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use common::{
    build::{BuildDef, StepDef, build_client::BuildClient},
    models::BuildDefinition,
};
use serde::Deserialize;
use tokio::fs;
use tonic::transport::Channel;
use tracing::{error, info};

type Result<T, E = ErrorResponse> = core::result::Result<T, E>;

struct ErrorResponse(anyhow::Error);

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        if let Some(error) = self.0.downcast_ref::<std::io::Error>() {
            match error.kind() {
                std::io::ErrorKind::NotFound => {
                    info!("{:#?}", self.0);
                    (StatusCode::NOT_FOUND, self.0.to_string()).into_response()
                }
                _ => {
                    error!("{:#?}", self.0);
                    (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
                }
            };
        }
        error!("{:#?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "an internal error has occured",
        )
            .into_response()
    }
}

impl<E> From<E> for ErrorResponse
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    path: String,
}

#[axum::debug_handler]
async fn build_handler(
    State(state): State<AppState>,
    Json(req): Json<BuildRequest>,
) -> Result<String> {
    let base_path = PathBuf::from(req.path);

    info!("Trying to find project at: {:?}", base_path);

    let build_file = base_path.join("ritning.yaml");

    let file_content = fs::read_to_string(build_file).await?;
    let definition: BuildDefinition = serde_yaml::from_str(&file_content)?;

    let steps: HashMap<String, StepDef> = definition
        .pipeline
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect();

    let build_def = BuildDef {
        steps,
        project_root: base_path.to_string_lossy().into(),
        env: definition.env,
    };

    let request = tonic::Request::new(build_def);

    let mut client = state.client.clone();
    let response = client.run(request).await?;

    let mut stream = response.into_inner();

    tokio::spawn(async move {
        while let Ok(Some(message)) = stream.message().await {
            info!("{:?}", message);
        }
    });

    Ok("Ok".to_owned())
}

#[derive(Clone)]
struct AppState {
    client: BuildClient<Channel>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = BuildClient::connect("http://[::1]:50051").await?;
    let state = AppState { client };

    let app = Router::new()
        .route("/build", post(build_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
