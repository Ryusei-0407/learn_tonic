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

fn echo_requests_iter() -> impl Stream<Item = EchoRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| EchoRequest {
        message: format!("msg {i:02}"),
    })
}

async fn call_bidirectional_stream_echo(client: &mut EchoClient<Channel>, num: usize) {
    let in_stream = echo_requests_iter().take(num);

    let response = client.bidirectional_stream_echo(in_stream).await.unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        println!("received message: `{}`", received.message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EchoClient::connect("http://127.0.0.1:50051").await?;

    call_unary_echo(&mut client, "Tonic").await;

    call_stream_echo(&mut client, 5).await;

    call_bidirectional_stream_echo(&mut client, 17).await;

    Ok(())
}
