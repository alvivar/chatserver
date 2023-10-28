// Properties

let _socket = null;
let _transmisionComplete = true;

const _maxReconnectionAttempts = 30;
const _maxReconnectionDelay = 60000; // 1 minute
const _baseReconnectionDelay = 1000; // 1 second

let _reconnectionAttempts = 0;
let _reconnectionDelay = _baseReconnectionDelay;
let _reconnectionTimer = 0;

let _reconLogId = -1;
let _reconLogTimer = 0;

const messages = document.getElementById('messages');
const pError = messages.querySelector('p#error');

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
		_socket.send(`${name} ${message}`);
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

function startReconLog(currentTime) {
	clearReconLog();

	_reconLogId = setInterval(() => {
		_reconLogTimer += 1;

		if (_reconnectionAttempts >= _maxReconnectionAttempts) {
			updateError(`Disconnected. Try refreshing the page.`);
			clearReconLog();
			return;
		}

		let time = currentTime / 1000 - _reconLogTimer;
		time = Math.max(0, Math.round(time));

		let message = `Disconnected. Reconnecting in  ${time} seconds...`;
		if (time === 0) {
			message = `Disconnected. Reconnecting...`;
		}

		updateError(message);
	}, 1000);
}

function clearReconLog() {
	clearInterval(_reconLogId);
	_reconLogTimer = 0;
}

// Main

connect();
