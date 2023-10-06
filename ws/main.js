(function () {
	if (!window.WebSocket) {
		alert('Your browser does not support WebSocket.');
		return;
	}

	const socket = new WebSocket('ws://127.0.0.1:8080/ws');

	socket.addEventListener('open', (event) => {
		log('Connected to the server.');
	});

	socket.addEventListener('message', (event) => {
		log(`${event.data}`);
	});

	socket.addEventListener('close', (event) => {
		log('Disconnected from the server.');
	});

	socket.addEventListener('error', (event) => {
		log('Error: ' + JSON.stringify(event));
	});

	document.getElementById('input').addEventListener('keydown', (event) => {
		if (event.key === 'Enter') sendMessage();
	});

	document.getElementById('send').addEventListener('click', (event) => {
		sendMessage();
	});

	function sendMessage() {
		const input = document.getElementById('input');
		const message = input.value;
		input.value = '';

		let name = document.getElementById('name').value;

		if (message.trim() !== '') {
			socket.send(`${name}${message}`);
		}
	}

	function messageHtml(message) {
		const p = document.createElement('p');
		p.classList.add('py-2');
		p.textContent = message;
		return p;
	}

	function log(text) {
		const messages = document.getElementById('messages');
		messages.insertBefore(messageHtml(text), messages.firstChild);
	}
})();
