use std::{path::PathBuf, sync::Arc};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use common::{
    build::{BuildDef, Context, StepDef, build_client::BuildClient},
    models::BuildDefinition,
    startup::registration_server::RegistrationServer,
};
use features::RegistrationController;
use serde::Deserialize;
use tokio::fs;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing::{error, info};

use crate::features::{AgentRepository, HealthcheckService};

mod features;

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
    State(_state): State<AppState>,
    Json(req): Json<BuildRequest>,
) -> Result<String> {
    let base_path = PathBuf::from(req.path);

    info!("Trying to find project at: {:?}", base_path);

    let build_file = base_path.join("ritning.yaml");

    let file_content = fs::read_to_string(build_file).await?;
    let definition: BuildDefinition = serde_yaml::from_str(&file_content)?;

    dbg!(&definition);

    let steps: Vec<StepDef> = definition
        .pipeline
        .into_iter()
        .map(|step| step.into())
        .collect();

    let build_def = BuildDef {
        steps,
        env: definition.env,
        context: Some(Context {
            project_root: base_path.to_string_lossy().into(),
        }),
    };

    let request = tonic::Request::new(build_def);

    let mut client = BuildClient::connect("http://[::1]:50052").await?;
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
struct AppState {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting up controller");

    let agent_repository = Arc::new(AgentRepository::new());
    let healthcheck_service = Arc::new(HealthcheckService::run(agent_repository.clone()));

    let controller = RegistrationController {
        agent_repository: agent_repository.clone(),
        healthcheck_service: healthcheck_service.clone(),
    };

    let server = RegistrationServer::new(controller);

    let addr = "[::1]:50051".parse()?;
    println!("Registrations are open on {addr}");

    let shutdown = CancellationToken::new();

    let grpc_shutdown = shutdown.clone();
    let http_shutdown = shutdown.clone();

    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(server)
            .serve_with_shutdown(addr, async move {
                grpc_shutdown.cancelled().await;
            })
            .await
            .unwrap();
    });

    let state = AppState {};

    let app = Router::new()
        .route("/build", post(build_handler))
        .with_state(state);

    let http_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                http_shutdown.cancelled().await;
            })
            .await
            .unwrap();
    });

    let ctrl_c = tokio::signal::ctrl_c();

    println!("Rest API is up and listening on 3000");
    tokio::select! {
        _ = ctrl_c => {
            println!("Ctrl+C received");
            shutdown.cancel();
        }
        _ = grpc_task => {
            eprintln!("gRPC exited → shutting down system");
            shutdown.cancel();
        }
        _ = http_task => {
            eprintln!("HTTP exited → shutting down system");
            shutdown.cancel();
        }
    }

    healthcheck_service.stop().await;

    Ok(())
}
