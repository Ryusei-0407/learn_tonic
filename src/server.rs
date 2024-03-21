use tonic::{transport::Server, Request, Response, Status};

use echo::echo_server::{Echo, EchoServer};
use echo::{EchoRequest, EchoResponse};

pub mod echo {
    tonic::include_proto!("grpc.example.echo");
}

#[derive(Debug, Default)]
pub struct MyEcho {}

#[tonic::async_trait]
impl Echo for MyEcho {
    #[tracing::instrument]
    async fn unary_echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        tracing::info!("call unary_echo from {:?}", request.remote_addr());

        let reply = echo::EchoResponse {
            message: format!("Recived message is: {}", request.into_inner().message),
        };

        tracing::debug!("sending response");

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .json()
        .init();

    let addr = "127.0.0.1:50051".parse().unwrap();
    let server = MyEcho::default();

    tracing::info!("Starting server listening on {addr}");

    Server::builder()
        .trace_fn(|_| tracing::info_span!("echo_server"))
        .add_service(EchoServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
