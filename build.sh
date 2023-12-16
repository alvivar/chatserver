#!/bin/bash

# Recreates a build directory and copy the necessary files to run the server as
# stand alone.

cargo build --release
cd web/
npm run build
cd ..

rm -rf build
mkdir -p build
cp .env build/
cp target/release/chatserver.exe build/

mkdir -p build/web
cp web/index.html build/web/
cp web/style.css build/web/
cp web/favicon.ico build/web/

mkdir -p build/web/js
cp web/js/main.js build/web/js/
cp web/js/pubsub.js build/web/js/
cp web/js/socket.js build/web/js/
cp web/js/ui.js build/web/js/
