import { RequestMessageSchema } from "@wired-protocol/types";
import { SctpStreamParameters } from "mediasoup/node/lib/SctpParameters";
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

    const { type, data } = parsed.data;

    switch (type) {
      case "join": {
        player.join(data);
        break;
      }

      case "leave": {
        player.leave(data);
        break;
      }

      case "send_chat_message": {
        player.chat(data);
        break;
      }

      case "set_grounded": {
        player.grounded = data;
        break;
      }

      case "set_name": {
        player.name = data;
        break;
      }

      case "set_avatar": {
        player.avatar = data;
        break;
      }

      case "set_address": {
        player.address = data;
        break;
      }

      // WebRTC
      case "webrtc_get_router_rtp_capabilities": {
        player.send({ type: "webrtc_rtp_capabilities", data: router.rtpCapabilities });
        break;
      }

      case "pause_audio": {
        player.setPaused(data);
        break;
      }

      case "webrtc_create_transport": {
        createWebRtcTransport(router, webRtcServer)
          .then(({ transport, params }) => {
            player.setTransport(data, transport);

            player.send({
              type: "webrtc_transport_created",
              data: { type: data, options: params },
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case "webrtc_connect_transport": {
        const transport =
          data.type === "producer" ? player.producerTransport : player.consumerTransport;
        if (!transport) break;

        transport.connect({ dtlsParameters: data.dtlsParameters });
        break;
      }

      case "webrtc_produce": {
        player.produce(data);
        break;
      }

      case "webrtc_produce_data": {
        if (data.streamId === undefined) {
          console.warn("Stream ID is undefined");
          break;
        }

        player.produceData(data as SctpStreamParameters);
        break;
      }

      case "set_rtp_capabilities": {
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
