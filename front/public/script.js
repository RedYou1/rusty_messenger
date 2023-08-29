let statusDiv = document.getElementById('status');

let STATUS = false;

// Add `message` from `username` to `room`. If `push`, then actually store the
// message. If the current room is `room`, render the message.
function addMessage(room, username, message) {
  let messagesDiv = document.getElementById('messages');

  let m = document.createElement("div");
  let u = document.createElement("span");
  let t = document.createElement("span");

  m.className = "message";
  u.className = "username";
  u.innerText = username;
  t.className = "text";
  t.innerText = message;

  m.appendChild(u);
  m.appendChild(t);
  messagesDiv.appendChild(m);
}

// Subscribe to the event source at `uri` with exponential backoff reconnect.
function subscribe(uri) {
  var retryTime = 1;

  function connect(uri) {
    const events = new EventSource(uri);

    events.addEventListener("message", (ev) => {
      console.log("raw data", JSON.stringify(ev.data));
      console.log("decoded data", JSON.stringify(JSON.parse(ev.data)));
      const msg = JSON.parse(ev.data);
      if (!"message" in msg || !"room" in msg || !"username" in msg) return;
      addMessage(msg.room, msg.username, msg.message, true);
    });

    events.addEventListener("open", () => {
      setConnectedStatus(true);
      console.log(`connected to event stream at ${uri}`);
      retryTime = 1;
    });

    events.addEventListener("error", () => {
      setConnectedStatus(false);
      events.close();

      let timeout = retryTime;
      retryTime = Math.min(64, retryTime * 2);
      console.log(`connection lost. attempting to reconnect in ${timeout}s`);
      setTimeout(() => connect(uri), (() => timeout * 1000)());
    });
  }

  connect(uri);
}

// Set the connection status: `true` for connected, `false` for disconnected.
function setConnectedStatus(status) {
  STATUS = status;
  statusDiv.className = (status) ? "connected" : "reconnecting";
}

subscribe("http://127.0.0.1:8000/events/0");