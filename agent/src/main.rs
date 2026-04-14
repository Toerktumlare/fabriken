use std::pin::Pin;

use anyhow::{Context, Result, anyhow};
use common::{
    build::{
        BuildDef, BuildUpdate,
        build_server::{Build, BuildServer},
    },
    parser::{DefaultPipelineProducer, PipelineProducer},
    runtime::{DefaultRuntime, Runtime},
    startup::{RegistrationRequest, registration_client::RegistrationClient},
};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct BuildController;

type BuildUpdateStream = Pin<Box<dyn Stream<Item = Result<BuildUpdate, Status>> + Send>>;

#[tonic::async_trait]
impl Build for BuildController {
    type RunStream = BuildUpdateStream;

    async fn run(&self, request: Request<BuildDef>) -> Result<Response<Self::RunStream>, Status> {
        info!("Got a request from {:?}", request.remote_addr());
        info!("{:#?}", request);

        let build_def = request.into_inner();
        info!("Constructing pipeline from received Build Definition");
        let (global_data, pipeline) = DefaultPipelineProducer::produce(build_def.into())
            .await
            .unwrap();

        info!("Starting up the runtime");
        let mut global_rx = DefaultRuntime::run(pipeline, global_data).await;

        let (grpc_tx, grpc_rx) = mpsc::channel(100);
        tokio::spawn(async move {
            while let Some(event) = global_rx.recv().await {
                let response_event = event.into();
                let res = Ok(response_event);
                grpc_tx.send(res).await.unwrap();
            }
        });

        let response_stream = ReceiverStream::new(grpc_rx);
        Ok(Response::new(Box::pin(response_stream) as Self::RunStream))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (_health_reporter, health_service) = tonic_health::server::health_reporter();

    let addr = "[::1]:50052".parse()?;
    let controller = BuildController;
    let server = BuildServer::new(controller);

    println!("Build Agent listening on {addr}");

    let (tx_ready, rx_ready) = oneshot::channel();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(server)
            .serve_with_shutdown(addr, async {
                tx_ready.send(()).unwrap();
                tokio::signal::ctrl_c().await.ok();
            })
            .await
    });

    rx_ready
        .await
        .context("Server did not start up correctly")?;

    let mut client = RegistrationClient::connect("http://[::1]:50051").await?;
    let response = client
        .register(RegistrationRequest {
            name: "hungry_hippo".to_string(),
            addr: addr.to_string(),
        })
        .await?
        .into_inner();

    if !response.success {
        error!("Could not register agent with controller");
        return Err(anyhow!("Could not register agent"));
    }

    info!("Agent registered. Press Ctrl+C to exit.");

    tokio::select! {
        res = handle => {
            match res {
                Ok(Ok(_)) => info!("Server exited naturally"),
                Ok(Err(e)) => error!("Server error: {}", e),
                Err(e) => error!("Server task panicked: {}", e),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received, shutting down...");
        }
    }

    Ok(())
}
