// Properties

let transmisionComplete = true;

// Sockets

const socket = new WebSocket('ws://127.0.0.1:8080/ws');

socket.addEventListener('open', () => {
	newChat('Connected to the server.');
});

socket.addEventListener('message', (event) => {
	if (event.data === '\0') {
		transmisionComplete = true;
		return;
	}

	if (transmisionComplete) {
		newChat(event.data);
		transmisionComplete = false;
	} else {
		appendText(event.data);
	}
});

socket.addEventListener('close', () => {
	newChat('Disconnected from the server.');
});

socket.addEventListener('error', (event) => {
	console.log(`Error: ${event}`);
});

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
		socket.send(`${name} ${message}`);
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