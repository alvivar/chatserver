import { PubSub } from "./pubsub.js";

const preprompt = document.getElementById("preprompt");
const input = document.getElementById("input");
const sendButton = document.getElementById("send");
const messages = document.getElementById("messages");
const pError = messages.querySelector("p#error");

const userBGColor = "bg-blue-200";
const systemBGColor = "bg-indigo-200";

let reconLogId = -1;
let reconLogTimer = 0;

const UIEvent = {
    sendMessage: "ui.sendMessage",
    copiedToClipboard: "ui.copiedToClipboard",
};

// Events

preprompt.addEventListener("keydown", (event) => {
    sendOnKeyPress(event);
    console.log("preprompt keydown");
});

input.addEventListener("keydown", (event) => {
    sendOnKeyPress(event);
});

preprompt.addEventListener("input", () => {
    adjustTextareaHeight(preprompt);
});

input.addEventListener("input", () => {
    adjustTextareaHeight(input);
});

sendButton.addEventListener("click", () => {
    sendMessage();
});

// Functions

function countWords(str) {
    return str.trim().split(/\s+/).length;
}

function sendOnKeyPress(event) {
    if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        sendMessage();
    }
}

function adjustTextareaHeight(textarea) {
    textarea.style.height = "4px"; // Temporarily shrink the height to force a scrollHeight recalculation.
    textarea.style.height = textarea.scrollHeight + 4 + "px";
}

function sendMessage() {
    const message = input.value.trim();

    input.value = "";
    input.style.height = preprompt.style.height;
    window.scrollTo(0, 0);

    let prompt = preprompt.value.trim();
    let complete_message = `${prompt} ${message}`.trim();

    if (complete_message !== "") {
        PubSub.publish(UIEvent.sendMessage, complete_message);
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
        let toCopy = p.innerHTML;

        // Here's a simple trick to retrieve the inner text of an element that
        // contains HTML entities. This trick essentially involves copying a
        // formatted text.
        var formatDiv = document.createElement("div");
        formatDiv.innerHTML = toCopy;
        toCopy = formatDiv.innerText;

        navigator.clipboard
            .writeText(toCopy)
            .then(() => {
                console.log(`Copied: ${toCopy}`);
                const words = countWords(toCopy);
                PubSub.publish(UIEvent.copiedToClipboard, words);
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
