import { RequestMessageSchema } from "@wired-protocol/types";
import uWS from "uWebSockets.js";

import { createMediasoupWorker, createWebRtcTransport } from "./mediasoup";
import { Player } from "./Player";
import { SpaceRegistry } from "./SpaceRegistry";
import { UserData, uWebSocket } from "./types";

const textDecoder = new TextDecoder();
const PORT = 4000;
const cert_file_name = process.env.SSL_CERT;
const key_file_name = process.env.SSL_KEY;

// Create WebSocket server
// Use SSL if cert and key are provided
const server =
  cert_file_name && key_file_name ? uWS.SSLApp({ key_file_name, cert_file_name }) : uWS.App();

// Create Mediasoup router
const { router, webRtcServer } = await createMediasoupWorker();

const spaces = new SpaceRegistry(server);
const players = new Map<uWebSocket, Player>();

// Handle WebSocket connections
server.ws<UserData>("/*", {
  compression: uWS.SHARED_COMPRESSOR,
  idleTimeout: 60,

  open: (ws) => {
    players.set(ws, new Player(ws, spaces));
  },

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

      case "xyz.unavi.world.user.grounded": {
        player.grounded = data;
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
          id: "xyz.unavi.webrtc.router.rtpCapabilities",
          data: router.rtpCapabilities,
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
              id: "xyz.unavi.webrtc.transport.created",
              data: { type: data, options: params },
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case "xyz.unavi.webrtc.transport.connect": {
        const transport =
          data.type === "producer" ? player.producerTransport : player.consumerTransport;
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
          streamId: data.streamId,
          maxPacketLifeTime: data.maxPacketLifeTime,
          maxRetransmits: data.maxRetransmits,
          ordered: data.ordered,
        });
        break;
      }

      case "xyz.unavi.webrtc.rtpCapabilities.set": {
        player.rtpCapabilities = data;
        break;
      }
    }
  },

  close: (ws) => {
    const player = players.get(ws);
    if (player) player.close();
    players.delete(ws);
  },
});

// Handle HTTP requests
server.get("/player-count/*:uri", (res, req) => {
  const uri = req.getUrl().slice(14);

  const space = spaces.getSpace(uri);
  const playerCount = space ? space.playerCount : 0;

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
