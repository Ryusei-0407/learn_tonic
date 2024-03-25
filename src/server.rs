use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

use echo::{EchoRequest, EchoResponse};

pub mod echo {
    tonic::include_proto!("grpc.example.echo");
}

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<EchoResponse, Status>> + Send>>;

#[derive(Debug, Default)]
pub struct EchoServer {}

#[tonic::async_trait]
impl echo::echo_server::Echo for EchoServer {
    #[tracing::instrument]
    async fn unary_echo(&self, req: Request<EchoRequest>) -> EchoResult<EchoResponse> {
        let reply = echo::EchoResponse {
            message: format!("message is: {}", req.into_inner().message),
        };

        Ok(Response::new(reply))
    }

    type ServerStreamEchoStream = ResponseStream;

    #[tracing::instrument]
    async fn server_stream_echo(
        &self,
        req: Request<EchoRequest>,
    ) -> EchoResult<Self::ServerStreamEchoStream> {
        tracing::info!("call server_streaming_echo");
        tracing::info!("client connected from: {:?}", req.remote_addr());

        let repeat = std::iter::repeat(EchoResponse {
            message: req.into_inner().message,
        });

        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        tracing::info!("Suucess received message");
                    }
                    Err(_) => {
                        tracing::info!("Closed connection");
                        break;
                    }
                }
            }
            tracing::info!("stream done");
        });

        let output_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(output_stream)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = "127.0.0.1:50051".to_socket_addrs().unwrap().next().unwrap();
    let server = EchoServer::default();

    tracing::info!("Starting server listening on {addr}");

    Server::builder()
        .trace_fn(|_| tracing::info_span!("echo_server"))
        .add_service(echo::echo_server::EchoServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
