import { MessageSchema } from "@wired-labs/protocol";
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
    const parsed = MessageSchema.toHost.safeParse(JSON.parse(text));

    if (!parsed.success) {
      console.warn(parsed.error);
      return;
    }

    const { subject, data } = parsed.data;

    switch (subject) {
      case "join": {
        player.join(data);
        break;
      }

      case "leave": {
        player.leave(data);
        break;
      }

      case "chat": {
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
      case "get_router_rtp_capabilities": {
        player.send({ subject: "router_rtp_capabilities", data: router.rtpCapabilities });
        break;
      }

      case "resume_audio": {
        player.resumeAudio();
        break;
      }

      case "create_transport": {
        createWebRtcTransport(router, webRtcServer)
          .then(({ transport, params }) => {
            player.setTransport(data.type, transport);

            player.send({
              subject: "transport_created",
              data: {
                type: data.type,
                options: {
                  id: params.id,
                  iceParameters: params.iceParameters,
                  iceCandidates: params.iceCandidates,
                  dtlsParameters: params.dtlsParameters,
                  sctpParameters: params.sctpParameters,
                },
              },
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case "connect_transport": {
        const transport =
          data.type === "producer" ? player.producerTransport : player.consumerTransport;
        if (!transport) break;

        transport.connect({ dtlsParameters: data.dtlsParameters });
        break;
      }

      case "produce": {
        player.produce(data);
        break;
      }

      case "produce_data": {
        if (data.streamId === undefined) {
          console.warn("Stream ID is undefined");
          break;
        }

        player.produceData(data as any);
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
server.get("/spaces/:id/player-count", (res, req) => {
  const idString = req.getParameter(0);
  const id = parseInt(idString);

  const space = spaces.getSpace(id);
  const playerCount = space ? space.playerCount : 0;

  console.info(`/spaces/${id}/player-count: ${playerCount}`);

  res.write(String(playerCount));
  res.writeStatus("200 OK");
  res.end();
});

// Start server
server.listen(PORT, (listenSocket) => {
  if (listenSocket) console.info(`✅ Listening to port ${PORT}`);
  else console.error(`❌ Failed to listen to port ${PORT}`);
});
