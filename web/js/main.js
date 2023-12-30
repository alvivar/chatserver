import { PubSub } from "./pubsub.js";
import { setAlert } from "./alert.js";
import { connect, send, canReconnect, SocketEvent } from "./socket.js";
import {
    appendText,
    newChat,
    updateError,
    startReconLog,
    clearReconLog,
    UIEvent,
} from "./ui.js";

// Subscriptions

PubSub.subscribe(SocketEvent.connected, () => {
    clearReconLog();
    updateError("Connected.");
});

PubSub.subscribe(SocketEvent.partialMessage, (message) => {
    appendText(message);
});

PubSub.subscribe(SocketEvent.completeMessage, (message) => {
    let parts = message.split(":");

    if (parts.length < 2) {
        newChat(message);
        return;
    }

    let command = parts[0].trim().toLowerCase();
    let value = parts.slice(1).join(":").trim();

    if (command.includes("alert")) {
        setAlert(value);
    } else if (command.includes("error")) {
        updateError(value);
    }
});

PubSub.subscribe(SocketEvent.reconnection, (delay) => {
    startReconLog(delay, canReconnect);
});

PubSub.subscribe(UIEvent.sendMessage, (message) => {
    send(message);
});

PubSub.subscribe(UIEvent.copiedToClipboard, (wordsCount) => {
    setAlert(`Copied ${wordsCount} words to clipboard!`);
});

// Functions

function welcome() {
    setAlert("Welcome!");
    document.removeEventListener("mousemove", welcome);
}

// Main

document.addEventListener("mousemove", welcome);
connect("ws://localhost:8080/ws");
