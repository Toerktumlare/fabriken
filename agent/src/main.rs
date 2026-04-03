use std::pin::Pin;

use anyhow::Result;
use common::{
    build::{
        BuildDef, BuildUpdate,
        build_server::{Build, BuildServer},
    },
    parser::{DefaultPipelineProducer, PipelineProducer},
    runtime::{DefaultRuntime, Runtime},
};
use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tracing::info;

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
        let pipeline = DefaultPipelineProducer::produce(build_def.into())
            .await
            .unwrap();
        let mut global_rx = DefaultRuntime::run(pipeline).await;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:50051".parse().unwrap();
    let controller = BuildController; // your struct implementing Build
    let server = BuildServer::new(controller);

    println!("GreeterServer listening on {addr}");

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
