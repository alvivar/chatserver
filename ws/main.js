(function () {
	if (!window.WebSocket) {
		alert('Your browser does not support WebSocket.');
		return;
	}

	const socket = new WebSocket('ws://127.0.0.1:8080');

	socket.addEventListener('open', (event) => {
		log('Connected to the server.');
	});

	socket.addEventListener('message', (event) => {
		log(`Server: ${event.data}`);
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

	function sendMessage() {
		const input = document.getElementById('input');

		const message = input.value;
		input.value = '';

		if (message.trim() !== '') {
			socket.send(message);
			log(`Sent: ${message}`);
		}
	}

	function messageHtml(message) {
		const p = document.createElement('p');
		p.textContent = message;
		return p;
	}

	function log(text) {
		const logs = document.getElementById('logs');
		logs.insertBefore(messageHtml(text), logs.firstChild);
	}
})();
