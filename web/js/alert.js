const alert = document.getElementById("alert");
const alertHeight = alert.offsetHeight;
let clearTimer = null;

function setAlert(message, duration = 3000) {
    alert.innerHTML = message;
    alert.classList.remove("hidden");

    clearTimeout(clearTimer);
    clearTimer = setTimeout(clearAlert, duration);
}

function clearAlert() {
    alert.classList.add("hidden");
}

function updateAlertPosition(event) {
    const x = event.pageX;
    const y = event.pageY;

    alert.style.left = `${x}px`;
    alert.style.top = `${y - alertHeight}px`;
}

// Main

alert.classList.add("hidden");
document.addEventListener("mousemove", updateAlertPosition);

// Exports

export { setAlert };
