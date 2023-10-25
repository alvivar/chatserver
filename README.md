# chatserver

Chat with OpenAI models on a WebSocket server using `fastwebsockets` in Rust, powered by Tokio & Hyper.

## Run on development mode

    cargo run --release

Runs the web client on `localhost:8080` and the WebSocket server on `localhost:8080/ws`.

## Simple Web

The web client is a simple HTML with a simple JavaScript that connects to the WebSocket server and sends messages to it.

The complicated part may be using `tailwindcss` to style the HTML :laughing:

### TailwindCSS

We are using the CLI version of `tailwindcss` to build the CSS file, if you made some changes to it, remember to,

install it with,

    npm install

and then just get inside the `web` folder and run,

    npx tailwindcss -o style.css --minify

or the shorthand.

    npm run build
