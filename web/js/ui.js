import { PubSub } from './pubsub.js';

const messages = document.getElementById('messages');
const pError = messages.querySelector('p#error');

let reconLogId = -1;
let reconLogTimer = 0;

const UIEvent = {
	sendMessage: 'ui.sendMessage',
};

// UI

document.getElementById('input').addEventListener('keydown', (event) => {
	if (event.key === 'Enter') sendMessage();
});

document.getElementById('send').addEventListener('click', () => {
	sendMessage();
});

function sendMessage() {
	const input = document.getElementById('input');
	const message = input.value.trim();
	input.value = '';

	let name = document.getElementById('name').value.trim();

	if (message !== '') {
		PubSub.publish(UIEvent.sendMessage, `${name} ${message}`);
	}
}

function messageHtml(message) {
	const p = document.createElement('p');
	p.classList.add('py-4');
	p.classList.add('whitespace-pre-line');
	p.textContent = message;
	return p;
}

function appendText(text) {
	messages.firstChild.appendChild(document.createTextNode(text));
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
		p.id = 'error';
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