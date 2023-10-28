const connect = () => {
	_socket = new WebSocket('ws://127.0.0.1:8080/ws');

	_socket.addEventListener('open', () => {
		clearReconLog();
		updateError('Connected.');

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
