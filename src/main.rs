use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use fastwebsockets::upgrade::upgrade;
use fastwebsockets::{FragmentCollector, OpCode, WebSocketError};
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Body, Request, Response};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use data::{Message, SharedState, State};
mod data;

async fn handle_ws(
    mut ws: FragmentCollector<Upgraded>,
    client_addr: SocketAddr,
    state: &SharedState,
) -> Result<(), WebSocketError> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    {
        let mut state = state.write().await;
        state.clients.insert(client_addr, tx);
    }

    println!("New connection with {}", client_addr);

    loop {
        tokio::select! {
            frame = ws.read_frame() => {
                let frame = frame?;

                match frame.opcode {
                    OpCode::Close => {
                        println!("Closing connection with {}", client_addr);
                        break;
                    }
                    OpCode::Text => {
                        let text = String::from_utf8(frame.payload.to_vec()).unwrap();
                        state.read().await.broadcast(&client_addr, Message::Text(text)).await;
                        ws.write_frame(frame).await?;
                    }
                    OpCode::Binary => {
                        state.read().await.broadcast(&client_addr, Message::Binary(frame.payload.to_vec())).await;
                        ws.write_frame(frame).await?;
                    }
                    _ => {}
                }
            },
            frame = rx.recv() => {
                if let Some(frame) = frame {
                    ws.write_frame(frame.to_frame()).await?;
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn request_handler(
    mut request: Request<Body>,
    address: SocketAddr,
    state: SharedState,
) -> Result<Response<Body>> {
    let uri = request.uri().path();

    match uri {
        "/ws" => {
            let (response, fut) = upgrade(&mut request)?;

            tokio::spawn(async move {
                let ws: FragmentCollector<Upgraded> =
                    fastwebsockets::FragmentCollector::new(fut.await.unwrap());

                handle_ws(ws, address, &state).await.unwrap();

                {
                    let mut state = state.write().await;
                    state.clients.remove(&address);
                }
            });

            Ok(response)
        }

        _ => {
            let response = Response::builder()
                .status(404)
                .body("Not found (404)".into())?;

            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    let state = Arc::new(RwLock::new(State {
        clients: HashMap::new(),
    }));

    loop {
        let (stream, address) = listener.accept().await?;
        let state = state.clone();

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(
                    stream,
                    service_fn(move |req| request_handler(req, address, state.clone())),
                )
                .with_upgrades()
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
