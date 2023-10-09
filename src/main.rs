mod data;

use anyhow::Result;
use data::{Message, SharedState, State};
use fastwebsockets::{upgrade::upgrade, FragmentCollector, OpCode, WebSocketError};
use hyper::{server::conn::Http, service::service_fn, upgrade::Upgraded, Body, Request, Response};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

async fn handle_ws(
    mut ws: FragmentCollector<Upgraded>,
    address: SocketAddr,
    state: &SharedState,
) -> Result<(), WebSocketError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);
    {
        let mut state = state.write().await;
        state.clients.insert(address, tx);
    }

    println!("New connection with {}", address);

    loop {
        tokio::select! {
            frame = ws.read_frame() => {
                let frame = frame?;
                match frame.opcode {
                    OpCode::Close => {
                        println!("Closing connection with {}", address);
                        break;
                    }
                    OpCode::Text => {
                        let text = String::from_utf8(frame.payload.to_vec()).unwrap();
                        state.read().await.broadcast(&address, Message::Text(text)).await;
                        ws.write_frame(frame).await?;
                    }
                    OpCode::Binary => {
                        state.read().await.broadcast(&address, Message::Binary(frame.payload.to_vec())).await;
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
            let (response, upgrade) = upgrade(&mut request)?;

            tokio::spawn(async move {
                let ws = FragmentCollector::new(upgrade.await.unwrap());

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
    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await?;

    println!("Listening on {}", address);

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
                    service_fn(move |request| request_handler(request, address, state.clone())),
                )
                .with_upgrades()
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
