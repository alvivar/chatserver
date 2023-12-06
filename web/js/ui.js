import { PubSub } from "./pubsub.js";

const messages = document.getElementById("messages");
const pError = messages.querySelector("p#error");

const userBGColor = "bg-blue-200";
const systemBGColor = "bg-indigo-200";

let reconLogId = -1;
let reconLogTimer = 0;

const UIEvent = {
    sendMessage: "ui.sendMessage",
};

const preprompt = document.getElementById("preprompt");
const input = document.getElementById("input");

// UI

input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") sendMessage();
});

input.addEventListener("input", () => {
    input.style.overflow = "hidden";
    input.style.height = "inherit";
    input.style.height = `${input.scrollHeight}px`;
});

preprompt.addEventListener("input", () => {
    preprompt.style.overflow = "hidden";
    preprompt.style.height = "inherit";
    preprompt.style.height = `${preprompt.scrollHeight}px`;
});

document.getElementById("send").addEventListener("click", () => {
    sendMessage();
});

function sendMessage() {
    const message = input.value.trim();
    input.value = "";
    input.style.height = "inherit";

    let prompt = preprompt.value.trim();

    if (message !== "") {
        PubSub.publish(UIEvent.sendMessage, `${prompt} ${message}`);
    }
}

function messageHtml(message) {
    const p = document.createElement("p");
    p.classList.add(userBGColor);
    p.classList.add("p-4");
    p.classList.add("my-2");
    p.classList.add("whitespace-pre-line");
    p.classList.add("rounded-lg");
    p.textContent = message;

    p.addEventListener("click", () => {
        let textToCopy = p.innerText;
        navigator.clipboard
            .writeText(textToCopy)
            .then(() => {
                console.log(`Copied: ${textToCopy}`);
            })
            .catch((err) => {
                console.error(`Error copying: ${err}`);
            });
    });

    return p;
}

function appendText(text) {
    messages.firstChild.appendChild(document.createTextNode(text));

    if (messages.firstChild.classList.contains(userBGColor))
        messages.firstChild.classList.remove(userBGColor);

    if (!messages.firstChild.classList.contains(systemBGColor))
        messages.firstChild.classList.add(systemBGColor);
}

function newChat(text) {
    messages.insertBefore(messageHtml(text), messages.firstChild);
}

function updateError(text) {
    if (pError) {
        pError.textContent = text;
        messages.insertBefore(pError, messages.firstChild);
    } else {
        const p = messageHtml(text);
        p.id = "error";
        messages.insertBefore(p, messages.firstChild);
    }
}

function startReconLog(currentTime, canReconnectCallback) {
    clearReconLog();

    reconLogId = setInterval(() => {
        reconLogTimer += 1;

        if (!canReconnectCallback()) {
            updateError(`Disconnected. Try refreshing the page.`);
            clearReconLog();
            return;
        }

        let time = currentTime / 1000 - reconLogTimer;
        time = Math.max(0, Math.round(time));

        let message = `Disconnected. Reconnecting in  ${time} seconds...`;
        if (time === 0) {
            message = `Disconnected. Reconnecting...`;
        }

        updateError(message);
    }, 1000);
}

function clearReconLog() {
    clearInterval(reconLogId);
    reconLogTimer = 0;
}

export {
    appendText,
    newChat,
    updateError,
    startReconLog,
    clearReconLog,
    UIEvent,
};
