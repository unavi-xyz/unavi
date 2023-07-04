import { RequestMessageSchema } from "@wired-protocol/types";
import uWS from "uWebSockets.js";

import { createMediasoupWorker, createWebRtcTransport } from "./mediasoup";
import { Player } from "./Player";
import { UserData, uWebSocket } from "./types";
import { WorldRegistry } from "./WorldRegistry";

const textDecoder = new TextDecoder();
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
  close: (ws) => {
    const player = players.get(ws);
    if (player) player.close();
    players.delete(ws);
  },
  compression: uWS.SHARED_COMPRESSOR,

  idleTimeout: 60,

  message: (ws, buffer) => {
    const player = players.get(ws);
    if (!player) return;

    const text = textDecoder.decode(buffer);
    const parsed = RequestMessageSchema.safeParse(JSON.parse(text));

    if (!parsed.success) {
      console.warn(parsed.error);
      return;
    }

    const { id, data } = parsed.data;

    switch (id) {
      case "xyz.unavi.world.join": {
        player.join(data);
        break;
      }

      case "xyz.unavi.world.leave": {
        player.leave(data);
        break;
      }

      case "xyz.unavi.world.chat.send": {
        player.chat(data);
        break;
      }

      case "xyz.unavi.world.user.falling": {
        player.falling = data;
        break;
      }

      case "xyz.unavi.world.user.name": {
        player.name = data;
        break;
      }

      case "xyz.unavi.world.user.avatar": {
        player.avatar = data;
        break;
      }

      case "xyz.unavi.world.user.handle": {
        player.handle = data;
        break;
      }

      // WebRTC
      case "xyz.unavi.webrtc.router.rtpCapabilities.get": {
        player.send({
          data: router.rtpCapabilities,
          id: "xyz.unavi.webrtc.router.rtpCapabilities",
        });
        break;
      }

      case "xyz.unavi.webrtc.audio.pause": {
        player.setPaused(data);
        break;
      }

      case "xyz.unavi.webrtc.transport.create": {
        createWebRtcTransport(router, webRtcServer)
          .then(({ transport, params }) => {
            player.setTransport(data, transport);

            player.send({
              data: { options: params, type: data },
              id: "xyz.unavi.webrtc.transport.created",
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case "xyz.unavi.webrtc.transport.connect": {
        const transport =
          data.type === "producer"
            ? player.producerTransport
            : player.consumerTransport;
        if (!transport) break;

        transport.connect({ dtlsParameters: data.dtlsParameters });
        break;
      }

      case "xyz.unavi.webrtc.produce": {
        player.produce(data);
        break;
      }

      case "xyz.unavi.webrtc.produceData": {
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

      case "xyz.unavi.webrtc.rtpCapabilities.set": {
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
