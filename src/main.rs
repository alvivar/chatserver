use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use fastwebsockets::{
    upgrade::{self, upgrade},
    FragmentCollector, OpCode, WebSocketError,
};

use anyhow::Result;
use futures::StreamExt;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    upgrade::Upgraded,
    Request, Response,
};
use tokio::{
    net::TcpListener,
    sync::{mpsc, RwLock},
};

use http_body_util::Empty;

use std::{
    collections::HashMap,
    env, fs,
    io::{self},
    net::SocketAddr,
    sync::Arc,
};

mod data;
use data::{Message, SharedState, State};

mod filemap; // Static files are served from here.
use filemap::{FileData, FileMap};

const SERVER: &str = "//server"; // This is used to identify server messages.

async fn request_handler(
    mut request: Request<Incoming>,
    address: SocketAddr,
    state: SharedState,
    static_files: Arc<HashMap<String, FileMap>>,
) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    let mut uri = request.uri().path();

    if uri == "/" {
        uri = "/index.html";
    }

    match uri {
        "/ws" => {
            let (response, fut) = upgrade(&mut request)?;

            tokio::spawn(async move {
                handle_ws(fut, address, &state).await.unwrap();

                {
                    let mut state = state.write().await;
                    state.clients.remove(&address);
                }
            });

            Ok(response)
        }

        _ => {
            if let Some(map) = static_files.get(uri) {
                // let response = serve_file(&map.data, map.mime_type).await;

                // Ok(response)
                Ok(Response::new(Empty::new()))
            } else {
                let response = Response::builder()
                    .status(404)
                    .body("Not found (404)")
                    .unwrap();

                Ok(Response::new(Empty::new()))
            }
        }
    }
}

async fn serve_file(data: &FileData, mime_type: &'static str) -> Result<Response<Empty<Bytes>>> {
    // let body = match data {
    //     FileData::Bytes(b) => Body::from(b.to_vec()),
    // };

    // let response = Response::builder()
    //     .status(200)
    //     .header("Content-Type", mime_type)
    //     .body(body)?;

    // Ok(response)

    Ok(Response::new(Empty::new()))
}

async fn handle_ws(
    fut: upgrade::UpgradeFut,
    address: SocketAddr,
    state: &SharedState,
) -> Result<(), WebSocketError> {
    let mut ws = FragmentCollector::new(fut.await.unwrap());

    let (tx, mut rx) = mpsc::channel(128);
    {
        let mut state = state.write().await;
        state.clients.insert(address, tx);
    }

    println!("{} New", address);

    let (openai_ws_tx, mut openai_ws_rx) = mpsc::channel::<String>(128);
    let client = Client::new();

    // Session data.

    let prompts: HashMap<SocketAddr, Vec<String>> = HashMap::new();
    let prompts = Arc::new(RwLock::new(prompts));
    let current_model: Arc<RwLock<String>> = Arc::new(RwLock::new("gpt-3.5-turbo-1106".into()));

    loop {
        tokio::select! {
            frame = ws.read_frame() => {
                let frame = frame?;
                match frame.opcode {
                    OpCode::Close => {
                        println!("{} Closed", address);
                        break;
                    }

                    OpCode::Text => {
                        let prompt = String::from_utf8(frame.payload.to_vec()).unwrap();
                        store_prompt(address, prompt.clone(), &prompts).await;

                        let message = Message::Text(prompt.clone());
                        ws.write_frame(message.to_frame()).await?;

                        let eof = Message::Text("\0".into());
                        ws.write_frame(eof.to_frame()).await?;

                        // Commands.

                        let commands = extract_commands(&prompt);

                        let model;
                        if let Some(to_model) = commands.get("model") {
                            model = to_model.clone();

                            {
                                let mut model = current_model.write().await;
                                *model = to_model.clone();
                            }

                            let message = Message::Text(format!("{} Alert: Model set to {}.", SERVER, to_model));
                            ws.write_frame(message.to_frame()).await?;

                            let eof = Message::Text("\0".into());
                            ws.write_frame(eof.to_frame()).await?;
                        }
                        else
                        {
                            model = current_model.read().await.clone() ;
                        }

                        tokio::spawn(process_openai_request(
                            prompt,
                            openai_ws_tx.clone(),
                            client.clone(),
                            model,
                        ));
                    }

                    _ => {}
                }
            },

            message = openai_ws_rx.recv() => {
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
    model: String,
) -> anyhow::Result<()> {
    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .max_tokens(512u16)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .role(Role::User)
            .build()?
            .into()])
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
                eprintln!("OpenAI Error: {}", err);

                openai_to_ws_tx
                    .send(format!("{} OpenAI Error: {}", SERVER, err))
                    .await
                    .unwrap();
            }
        }
    }

    // This means that the transmission is over.
    openai_to_ws_tx.send("\0".into()).await.unwrap();

    Ok(())
}

async fn store_prompt(
    address: SocketAddr,
    prompt: String,
    prompts: &Arc<RwLock<HashMap<SocketAddr, Vec<String>>>>,
) {
    let mut prompts = prompts.write().await;
    let prompts = prompts.entry(address).or_insert(Vec::new());
    prompts.push(prompt);
}

async fn get_prompts(
    address: SocketAddr,
    prompts: Arc<RwLock<HashMap<SocketAddr, Vec<String>>>>,
    num_prompts: usize,
) -> Vec<String> {
    let prompts = prompts.read().await;

    if let Some(prompt_list) = prompts.get(&address) {
        let len = prompt_list.len();

        // If there are fewer prompts than requested, return all of them.
        if len <= num_prompts {
            return prompt_list.clone();
        }

        // Otherwise, return the last `num_prompts`.
        return prompt_list[len - num_prompts..].to_vec();
    }

    // Return an empty Vec if no prompts are found for the address.
    Vec::new()
}

fn extract_commands(input: &str) -> HashMap<String, String> {
    let mut commands = HashMap::new();
    let mut iter = input.split_whitespace().peekable();

    while let Some(word) = iter.next() {
        if word.starts_with('!') && word.len() > 1 {
            if let Some(&next_word) = iter.peek() {
                let command = &word[1..];
                commands.insert(command.to_string(), next_word.to_string());
            }
            iter.next(); // Skip the next word as it is already used as a value.
        }
    }

    commands
}

fn set_env_from_file(file_path: &str) -> io::Result<()> {
    let contents = fs::read_to_string(file_path)?;

    for line in contents.lines() {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            env::set_var(key, value);
        } else {
            eprintln!("Warning: Skipping invalid line: {}", line);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_env_from_file(".env")?;

    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await?;

    println!("{} Listening", address);

    let state = Arc::new(RwLock::new(State {
        clients: HashMap::new(),
    }));

    let static_files = FileMap::static_files();

    loop {
        let (stream, address) = listener.accept().await?;
        let state = state.clone();
        let static_files = static_files.clone();

        tokio::task::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |request| {
                        request_handler(request, address, state.clone(), static_files.clone())
                    }),
                )
                .with_upgrades()
                .await
            {
                eprintln!("Serve Connection Error: {:?}", err);
            }
        });
    }
}
