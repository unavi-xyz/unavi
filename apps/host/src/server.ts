import { ToHostMessage } from "@wired-labs/engine";
import uWS from "uWebSockets.js";

import { Players } from "./Players";

const textDecoder = new TextDecoder();
const PORT = parseInt(process.env.PORT || "4000");
const cert_file_name = process.env.SSL_CERT;
const key_file_name = process.env.SSL_KEY;

// Create WebSocket server
// Use SSL if cert and key are provided
const server =
  cert_file_name && key_file_name
    ? uWS.SSLApp({ key_file_name, cert_file_name })
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

      case "location": {
        players.publishLocation(ws, message.data);
        break;
      }

      case "message": {
        players.publishMessage(ws, message.data);
        break;
      }

      case "falling_state": {
        players.publishFallingState(ws, message.data);
        break;
      }
    }
  },

  close: (ws) => {
    players.removePlayer(ws);
  },
});

// Handle HTTP requests
server.get("/playercount/*", (res, req) => {
  const id = String(req.getUrl().slice(13));
  const playerCount = players.getPlayerCount(id);

  console.info(`🔢 Player count for ${id}: ${playerCount}`);

  res.write(String(playerCount));
  res.writeStatus("200 OK");
  res.end();
});

// Start server
server.listen(PORT, (listenSocket) => {
  if (listenSocket) console.info(`✅ Listening to port ${PORT}`);
  else console.error(`❌ Failed to listen to port ${PORT}`);
});
