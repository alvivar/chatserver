import { PubSub } from "./pubsub.js";
import { connect, send, canReconnect, SocketEvent } from "./socket.js";
import {
    appendText,
    newChat,
    updateError,
    startReconLog,
    clearReconLog,
    UIEvent,
} from "./ui.js";

PubSub.subscribe(SocketEvent.connected, () => {
    clearReconLog();
    updateError("Connected.");
});

PubSub.subscribe(SocketEvent.partialMessage, (message) => {
    appendText(message);
});

PubSub.subscribe(SocketEvent.completeMessage, (message) => {
    newChat(message);
});

PubSub.subscribe(SocketEvent.reconnection, (delay) => {
    startReconLog(delay, canReconnect);
});

PubSub.subscribe(UIEvent.sendMessage, (message) => {
    send(message);
});

connect("ws://127.0.0.1:8080/ws");
