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
    async fn unary_echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let reply = echo::EchoResponse {
            message: format!("Recived message is: {}", request.into_inner().message),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let server = MyEcho::default();

    println!("Server listening on {addr}");

    Server::builder()
        .add_service(EchoServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
