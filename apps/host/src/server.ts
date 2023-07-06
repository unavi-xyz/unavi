import { RequestMessageSchema } from "@wired-protocol/types";
import uWS from "uWebSockets.js";

import { createMediasoupWorker, createWebRtcTransport } from "./mediasoup";
import { Player } from "./Player";
import { UserData, uWebSocket } from "./types";
import { parseMessage } from "./utils/parseMessage";
import { WorldRegistry } from "./WorldRegistry";

const PORT = 4000;
const cert_file_name = process.env.SSL_CERT;
const key_file_name = process.env.SSL_KEY;

// Create WebSocket server
// Use SSL if cert and key are provided
const server =
  cert_file_name && key_file_name
    ? uWS.SSLApp({ cert_file_name, key_file_name })
    : uWS.App();

// Create Mediasoup router
const { router, webRtcServer } = await createMediasoupWorker();

const worlds = new WorldRegistry(server);
const players = new Map<uWebSocket, Player>();

// Handle WebSocket connections
server.ws<UserData>("/*", {
  close: (ws, code) => {
    console.info("Closing connection", code);

    const player = players.get(ws);
    if (player) player.close();
    players.delete(ws);
  },

  compression: uWS.SHARED_COMPRESSOR,
  idleTimeout: 60,
  maxPayloadLength: 16 * 1024 * 1024, // 16 MB

  message: (ws, buffer) => {
    const player = players.get(ws);
    if (!player) return;

    const message = parseMessage(buffer);
    if (!message) return;

    // Relay client messages to other players in the same world
    if (message.target === "client") {
      console.log("relaying len", buffer.byteLength);
      player.worlds.forEach((world) => {
        ws.publish(world.topic, buffer);
      });
      return;
    }

    const request = RequestMessageSchema.safeParse(message);

    if (!request.success) {
      console.warn(request.error);
      return;
    }

    const { data, id } = request.data;

    switch (id) {
      case "com.wired-protocol.world.join": {
        player.join(data);
        break;
      }

      case "com.wired-protocol.world.leave": {
        player.leave(data);
        break;
      }

      case "com.wired-protocol.world.chat.send": {
        player.chat(data);
        break;
      }

      case "com.wired-protocol.world.user.falling": {
        player.falling = data;
        break;
      }

      case "com.wired-protocol.world.user.name": {
        player.name = data;
        break;
      }

      case "com.wired-protocol.world.user.avatar": {
        player.avatar = data;
        break;
      }

      case "com.wired-protocol.world.user.handle": {
        player.handle = data;
        break;
      }

      case "com.wired-protocol.webrtc.router.rtpCapabilities.get": {
        player.send({
          data: router.rtpCapabilities,
          id: "com.wired-protocol.webrtc.router.rtpCapabilities",
        });
        break;
      }

      case "com.wired-protocol.webrtc.audio.pause": {
        player.setPaused(data);
        break;
      }

      case "com.wired-protocol.webrtc.transport.create": {
        createWebRtcTransport(router, webRtcServer)
          .then(({ transport, params }) => {
            player.setTransport(data, transport);

            player.send({
              data: { options: params, type: data },
              id: "com.wired-protocol.webrtc.transport.created",
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case "com.wired-protocol.webrtc.transport.connect": {
        const transport =
          data.type === "producer"
            ? player.producerTransport
            : player.consumerTransport;
        if (!transport) break;

        transport.connect({ dtlsParameters: data.dtlsParameters });
        break;
      }

      case "com.wired-protocol.webrtc.produce": {
        player.produce(data);
        break;
      }

      case "com.wired-protocol.webrtc.produceData": {
        if (data.streamId === undefined) {
          console.warn("Stream ID is undefined");
          break;
        }

        player.produceData({
          maxPacketLifeTime: data.maxPacketLifeTime,
          maxRetransmits: data.maxRetransmits,
          ordered: data.ordered,
          streamId: data.streamId,
        });
        break;
      }

      case "com.wired-protocol.webrtc.rtpCapabilities.set": {
        player.rtpCapabilities = data;
        break;
      }
    }
  },

  open: (ws) => {
    players.set(ws, new Player(ws, worlds));
  },
});

// Handle HTTP requests
server.get("/player-count/*:uri", (res, req) => {
  const uri = req.getUrl().slice(14);

  const world = worlds.getWorld(uri);
  const playerCount = world ? world.playerCount : 0;

  console.info(`/player-count/${uri}: ${playerCount}`);

  res.write(String(playerCount));
  res.writeStatus("200 OK");
  res.end();
});

// Start server
server.listen(PORT, (listenSocket) => {
  if (listenSocket) console.info(`✅ Listening to port ${PORT}`);
  else console.error(`❌ Failed to listen to port ${PORT}`);
});
