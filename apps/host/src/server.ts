import { ToHostMessage } from "@wired-labs/engine";
import uWS from "uWebSockets.js";

import { Players } from "./Players";

const PORT = 4000;
const textDecoder = new TextDecoder();

// Create WebSocket server
const server =
  process.env.NODE_ENV === "production"
    ? uWS.SSLApp({
        key_file_name: "misc/key.pem",
        cert_file_name: "misc/cert.pem",
      })
    : uWS.App();

// Create player manager
const players = new Players(server);

// Handle WebSocket connections
server.ws("/*", {
  compression: uWS.SHARED_COMPRESSOR,
  idleTimeout: 60,

  open: (ws) => {
    players.addPlayer(ws);
  },

  message: (ws, buffer) => {
    const text = textDecoder.decode(buffer);
    const message: ToHostMessage = JSON.parse(text);

    switch (message.subject) {
      case "join": {
        players.joinSpace(ws, message.data);
        break;
      }
      case "leave": {
        players.leaveSpace(ws);
        break;
      }
    }
  },

  close: (ws) => {
    players.removePlayer(ws);
  },
});

// Start server
server.listen(PORT, (listenSocket) => {
  if (listenSocket) console.info(`✅ Listening to port ${PORT}`);
  else console.error(`❌ Failed to listen to port ${PORT}`);
});
