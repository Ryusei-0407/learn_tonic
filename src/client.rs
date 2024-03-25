use echo::{echo_client::EchoClient, EchoRequest};
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

pub mod echo {
    tonic::include_proto!("grpc.example.echo");
}

async fn call_unary_echo(client: &mut EchoClient<Channel>, message: &str) {
    let request = tonic::Request::new(EchoRequest {
        message: message.into(),
    });

    let response = client.unary_echo(request).await.unwrap();

    println!("RESPONSE: {response:?}");
}

async fn call_stream_echo(client: &mut EchoClient<Channel>, num: usize) {
    let stream = client
        .server_stream_echo(EchoRequest {
            message: "foo".into(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut stream = stream.take(num);

    while let Some(item) = stream.next().await {
        println!("\treceived: {}", item.unwrap().message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EchoClient::connect("http://127.0.0.1:50051").await?;

    call_unary_echo(&mut client, "Tonic").await;

    call_stream_echo(&mut client, 10).await;

    Ok(())
}
