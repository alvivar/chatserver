use std::collections::HashMap;
use std::sync::Arc;

use fastwebsockets::upgrade;
use fastwebsockets::Frame;
use fastwebsockets::OpCode;
use fastwebsockets::Payload;
use fastwebsockets::WebSocket;
use fastwebsockets::WebSocketError;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

async fn handle_client(
    fut: upgrade::UpgradeFut,
    tx: mpsc::Sender<(usize, Frame<'_>)>,
) -> Result<(), WebSocketError> {
    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

    loop {
        let frame = ws.read_frame().await?;
        match frame.opcode {
            OpCode::Close => break,
            OpCode::Text | OpCode::Binary => {
                // ws.write_frame(frame).await?;
                // tx.send(frame).await.unwrap();
            }
            _ => {}
        }
    }

    println!("Client disconnected");

    Ok(())
}

async fn server_upgrade(
    mut req: Request<Body>,
    tx: mpsc::Sender<(usize, Frame<'_>)>,
) -> Result<Response<Body>, WebSocketError> {
    let (response, fut) = upgrade::upgrade(&mut req)?;

    let tx_clone = tx.clone();
    tokio::task::spawn(async move {
        if let Err(e) = tokio::task::unconstrained(handle_client(fut, tx_clone)).await {
            eprintln!("Error in websocket connection: {}", e);
        }
    });

    println!("Upgraded to websocket connection");

    Ok(response)
}

async fn broadcast(
    mut rx: mpsc::Receiver<(usize, Frame<'_>)>,
    clients: Arc<Mutex<HashMap<usize, WebSocket<TcpStream>>>>,
) {
    while let Some((id, frame)) = rx.recv().await {
        for (&client_id, client) in clients.lock().await.iter_mut() {
            let payload = Payload::from(frame.payload.to_vec());
            let frame = Frame::new(frame.fin, frame.opcode, None, payload);

            if id != client_id {
                let _ = client.write_frame(frame).await;
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), WebSocketError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server started, listening on 127.0.0.1:8080");

    let clients = Arc::new(Mutex::new(HashMap::<usize, WebSocket<TcpStream>>::new()));
    let (tx, rx) = mpsc::channel::<(usize, fastwebsockets::Frame)>(32);

    let clients_broadcast = clients.clone();
    tokio::spawn(async move {
        broadcast(rx, clients_broadcast).await;
    });

    loop {
        let (stream, _) = listener.accept().await?;

        println!("Client connected");

        let service = {
            let service_tx = tx.clone(); // Clone here
            service_fn(move |req: Request<Body>| server_upgrade(req, service_tx.clone()))
            // Clone again to make sure each request gets its own.
        };

        tokio::spawn(async move {
            let conn_fut = Http::new()
                .serve_connection(stream, service)
                .with_upgrades();
            if let Err(e) = conn_fut.await {
                eprintln!("An error occurred: {:?}", e);
            }
        });
    }
}
