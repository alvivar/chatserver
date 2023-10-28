// Properties

let _socket = null;
let _transmisionComplete = true;

const _maxReconnectionAttempts = 5;
const _maxReconnectionDelay = 60000; // 1 minute
const _baseReconnectionDelay = 1000; // 1 second

let _reconnectionAttempts = 0;
let _reconnectionDelay = _baseReconnectionDelay;
let _reconnectionTimer = 0;

let _reconnectTimerId = -1;
let _reconnectTimer = 0;

// Sockets

const connect = () => {
	_socket = new WebSocket('ws://127.0.0.1:8080/ws');

	_socket.addEventListener('open', () => {
		clearReconLog();
		updateError('Connected to the server.');

		_reconnectionAttempts = 0;
		_reconnectionDelay = _baseReconnectionDelay;
	});

	_socket.addEventListener('message', (event) => {
		if (event.data === '\0') {
			_transmisionComplete = true;
			return;
		}

		if (_transmisionComplete) {
			newChat(event.data);

			_transmisionComplete = false;
		} else {
			appendText(event.data);
		}
	});

	_socket.addEventListener('close', () => {
		if (_reconnectionAttempts < _maxReconnectionAttempts) {
			setTimeout(() => {
				connect();
			}, _reconnectionDelay);

			startReconLog(_reconnectionDelay);

			_reconnectionAttempts += 1;
			_reconnectionDelay = Math.min(
				_maxReconnectionDelay,
				_reconnectionDelay * 1.5
			);
		}
	});

	_socket.addEventListener('error', (event) => {
		console.debug(`Error: ${event}`);
	});
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
	const messages = document.getElementById('messages');
	messages.firstChild.appendChild(document.createTextNode(text));
}

function newChat(text) {
	const messages = document.getElementById('messages');
	messages.insertBefore(messageHtml(text), messages.firstChild);
}

function updateError(text) {
	const messages = document.getElementById('messages');
	const pError = messages.querySelector('p#error');

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

	_reconnectTimerId = setInterval(() => {
		_reconnectTimer += 1;

		if (_reconnectionAttempts >= _maxReconnectionAttempts) {
			updateError(`Disconnected.`);
			clearReconLog();
			return;
		}

		let time = currentTime / 1000 - _reconnectTimer;
		time = Math.max(0, Math.round(time));

		let message = `Disconnected. Reconnecting in  ${time} seconds...`;
		if (time === 0) {
			message = `Disconnected. Reconnecting...`;
		}

		updateError(message);
	}, 1000);
}

function clearReconLog() {
	clearInterval(_reconnectTimerId);
	_reconnectTimer = 0;
}

// Main

connect();
