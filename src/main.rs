use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use fastwebsockets::{upgrade::upgrade, FragmentCollector, OpCode, WebSocketError};
use futures::StreamExt;
use hyper::{server::conn::Http, service::service_fn, upgrade::Upgraded, Body, Request, Response};
use tokio::{
    net::TcpListener,
    sync::{mpsc, RwLock},
};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

mod data;
use data::{Message, SharedState, State};

#[derive(Clone, Debug)]
enum FileData {
    Str(&'static str),
    Bytes(&'static [u8]),
}

struct FileMap {
    data: FileData,
    mime_type: &'static str,
}

async fn request_handler(
    mut request: Request<Body>,
    address: SocketAddr,
    state: SharedState,
    filemap: Arc<HashMap<&str, FileMap>>,
) -> Result<Response<Body>> {
    let mut uri = request.uri().path();

    if uri == "/" {
        uri = "/index.html";
    }

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
            if let Some(map) = filemap.get(uri) {
                let response = serve_file(&map.data, map.mime_type).await?;

                Ok(response)
            } else {
                let response = Response::builder()
                    .status(404)
                    .body("Not found (404)".into())?;

                Ok(response)
            }
        }
    }
}

async fn serve_file(data: &FileData, mime_type: &'static str) -> Result<Response<Body>> {
    let body = match data {
        FileData::Str(s) => Body::from(*s),
        FileData::Bytes(b) => Body::from(b.to_vec()),
    };

    let response = Response::builder()
        .status(200)
        .header("Content-Type", mime_type)
        .body(body)?;

    Ok(response)
}

async fn handle_ws(
    mut ws: FragmentCollector<Upgraded>,
    address: SocketAddr,
    state: &SharedState,
) -> Result<(), WebSocketError> {
    let (tx, mut rx) = mpsc::channel(128);
    {
        let mut state = state.write().await;
        state.clients.insert(address, tx);
    }

    println!("New {}", address);

    let (openai_to_ws_tx, mut openai_to_ws_rx) = mpsc::channel::<String>(128);
    let client = Client::new();

    loop {
        tokio::select! {
            frame = ws.read_frame() => {
                let frame = frame?;
                match frame.opcode {
                    OpCode::Close => {
                        println!("Closing {}", address);
                        break;
                    }
                    OpCode::Text => {
                        let prompt = String::from_utf8(frame.payload.to_vec()).unwrap();

                        let message = Message::Text(prompt.clone());
                        ws.write_frame(message.to_frame()).await?;

                        let eof = Message::Text("\0".into());
                        ws.write_frame(eof.to_frame()).await?;

                        tokio::spawn(process_openai_request(
                            prompt,
                            openai_to_ws_tx.clone(),
                            client.clone(),
                        ));
                    }
                    _ => {}
                }
            },
            message = openai_to_ws_rx.recv() => {
                if let Some(message) = message {
                    let message = Message::Text(message);
                    ws.write_frame(message.to_frame()).await?;
                } else {
                    break;
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

async fn process_openai_request(
    prompt: String,
    openai_to_ws_tx: mpsc::Sender<String>,
    client: Client<OpenAIConfig>,
) -> anyhow::Result<()> {
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .max_tokens(512u16)
        .messages([ChatCompletionRequestMessageArgs::default()
            .content(&prompt)
            .role(Role::User)
            .build()?])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                    if let Some(ref content) = chat_choice.delta.content {
                        openai_to_ws_tx.send(content.clone()).await.unwrap();
                    }
                }
            }
            Err(err) => {
                println!("OpenAI Error: {}", err);

                openai_to_ws_tx
                    .send(format!("OpenAI Error: {}", err))
                    .await
                    .unwrap();
            }
        }
    }

    // This means that the transmission is over.
    openai_to_ws_tx.send("\0".into()).await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await?;

    println!("Listening {}", address);

    let state = Arc::new(RwLock::new(State {
        clients: HashMap::new(),
    }));

    let filemap: HashMap<&str, FileMap> = [
        (
            "/index.html",
            FileMap {
                data: FileData::Str(include_str!("../web/index.html")),
                mime_type: "text/html",
            },
        ),
        (
            "/main.js",
            FileMap {
                data: FileData::Str(include_str!("../web/main.js")),
                mime_type: "application/javascript",
            },
        ),
        (
            "/style.css",
            FileMap {
                data: FileData::Str(include_str!("../web/style.css")),
                mime_type: "text/css",
            },
        ),
        (
            "/favicon.ico",
            FileMap {
                data: FileData::Bytes(include_bytes!("../web/favicon.ico")),
                mime_type: "image/x-icon",
            },
        ),
    ]
    .into();

    let filemap = Arc::new(filemap);

    loop {
        let (stream, address) = listener.accept().await?;
        let state = state.clone();
        let filemap = filemap.clone();

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(
                    stream,
                    service_fn(move |request| {
                        request_handler(request, address, state.clone(), filemap.clone())
                    }),
                )
                .with_upgrades()
                .await
            {
                println!("Serve Connection Error: {:?}", err);
            }
        });
    }
}
