// Ok, my goal is this. I want that every websocket connection can have his own
// dedicated OpenAI, this way, every connected client could send a prompt and
// receive a response from OpenAI. At the same time.

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
    sync::{mpsc, RwLock, Semaphore},
};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

mod data;
use data::{Message, SharedState, State};

async fn handle_ws(
    mut ws: FragmentCollector<Upgraded>,
    address: SocketAddr,
    state: &SharedState,
    ws_to_openai_tx: mpsc::Sender<String>,
) -> Result<(), WebSocketError> {
    let (tx, mut rx) = mpsc::channel(128);
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
                        ws_to_openai_tx.send(text).await.unwrap();
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
    ws_to_openai_tx: mpsc::Sender<String>,
) -> Result<Response<Body>> {
    let uri = request.uri().path();

    match uri {
        "/ws" => {
            let (response, upgrade) = upgrade(&mut request)?;

            tokio::spawn(async move {
                let ws = FragmentCollector::new(upgrade.await.unwrap());

                handle_ws(ws, address, &state, ws_to_openai_tx)
                    .await
                    .unwrap();

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

async fn process_openai_request(
    text: String,
    openai_to_ws_tx: mpsc::Sender<String>,
    client: Client<OpenAIConfig>,
) -> anyhow::Result<()> {
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(512u16)
        .messages([ChatCompletionRequestMessageArgs::default()
            .content(&text)
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
                println!("Error with OpenAI: {}", err);
                // Optionally, send error to WebSocket client
                openai_to_ws_tx
                    .send(format!("Error: {}", err))
                    .await
                    .unwrap();
            }
        }
    }
    Ok(())
}

async fn openai_handler(
    mut ws_to_openai_rx: mpsc::Receiver<String>,
    openai_to_ws_tx: mpsc::Sender<String>,
    client: Client<OpenAIConfig>,
) -> anyhow::Result<()> {
    // This semaphore limits concurrent requests to a certain number, adjust as needed
    let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent requests

    while let Some(text) = ws_to_openai_rx.recv().await {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Acquire semaphore");

        let tx = openai_to_ws_tx.clone();
        let client = client.clone();

        tokio::spawn(async move {
            let _ = process_openai_request(text, tx, client).await;
            drop(permit); // release the semaphore permit
        });
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await?;

    println!("Listening on {}", address);

    let state = Arc::new(RwLock::new(State {
        clients: HashMap::new(),
    }));

    // Assuming you've initialized the channels and the client before this point:
    let (ws_to_openai_tx, ws_to_openai_rx) = mpsc::channel(128);
    let (openai_to_ws_tx, openai_to_ws_rx) = mpsc::channel(128);
    let client = Client::new(); // Or however you initialize the client

    // Start the OpenAI handler
    tokio::spawn(openai_handler(
        ws_to_openai_rx,
        openai_to_ws_tx,
        client.clone(),
    ));

    loop {
        let (stream, address) = listener.accept().await?;
        let state = state.clone();
        let ws_to_openai_tx = ws_to_openai_tx.clone();

        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(
                    stream,
                    service_fn(move |request| {
                        request_handler(request, address, state.clone(), ws_to_openai_tx.clone())
                    }),
                )
                .with_upgrades()
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

use fastwebsockets::Frame;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc::Sender, RwLock};

pub type Tx = Sender<Message>;
pub type SharedState = Arc<RwLock<State>>;

pub struct State {
    pub clients: HashMap<SocketAddr, Tx>,
}

impl State {
    pub async fn broadcast(&self, sender: &SocketAddr, msg: Message) {
        for (addr, tx) in self.clients.iter() {
            if addr != sender {
                tx.send(msg.clone()).await.unwrap();
            }
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Pong(Vec<u8>),
    Close(u16, String), // Code, Reason
}

impl Message {
    pub fn to_frame(&self) -> Frame {
        match self {
            Message::Text(text) => Frame::text(text.as_bytes().into()),
            Message::Binary(data) => Frame::binary(data.as_slice().into()),
            Message::Pong(data) => Frame::pong(data.as_slice().into()),
            Message::Close(code, reason) => Frame::close(*code, reason.as_bytes()),
        }
    }
}
