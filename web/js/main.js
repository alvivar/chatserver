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

const SERVER = "//server"; // Token to detect server messages.

// Subscriptions

PubSub.subscribe(SocketEvent.connected, () => {
    clearReconLog();
    updateError("Connected.");
});

PubSub.subscribe(SocketEvent.partialMessage, (message) => {
    appendText(message);
});

PubSub.subscribe(SocketEvent.completeMessage, (message) => {
    let words = message.trim().split(" ");

    console.log(words);
    console.log(words[0]);
    let isServerMessage = words[0] === SERVER;
    console.log(isServerMessage);

    if (isServerMessage) {
        let parts = message.split(":");
        let value = parts.slice(1).join(":").trim();

        setAlert(value);
        updateError(value);
    }

    if (!isServerMessage) {
        newChat(message);
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
