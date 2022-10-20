import { ToHostMessage } from "@wired-labs/engine";
import uWS from "uWebSockets.js";

import { createMediasoupRouter, createWebRtcTransport } from "./mediasoup";
import { Players } from "./Players";
import { send } from "./utils/send";

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

// Create Mediasoup router
const router = await createMediasoupRouter();

// Handle WebSocket connections
server.ws("/*", {
  compression: uWS.SHARED_COMPRESSOR,
  idleTimeout: 60,

  open: (ws) => {
    players.addPlayer(ws);
  },

  message: async (ws, buffer) => {
    const text = textDecoder.decode(buffer);
    const { subject, data }: ToHostMessage = JSON.parse(text);

    switch (subject) {
      case "join": {
        players.joinSpace(ws, data);
        break;
      }

      case "leave": {
        players.leaveSpace(ws);
        break;
      }

      case "location": {
        players.publishLocation(ws, data);
        break;
      }

      case "message": {
        players.publishMessage(ws, data);
        break;
      }

      case "falling_state": {
        players.publishFallingState(ws, data);
        break;
      }

      case "set_name": {
        players.publishName(ws, data);
        break;
      }

      case "set_avatar": {
        players.publishAvatar(ws, data);
        break;
      }

      case "set_handle": {
        players.publishHandle(ws, data);
        break;
      }

      // WebRTC
      case "get_router_rtp_capabilities": {
        send(ws, {
          subject: "router_rtp_capabilities",
          data: router.rtpCapabilities,
        });
        break;
      }

      case "create_transport": {
        const { transport, params } = await createWebRtcTransport(router);
        players.setTransport(ws, transport, data.type);

        const iceCandidates: any = params.iceCandidates.filter((candidate) => {
          return candidate.tcpType !== undefined;
        });

        send(ws, {
          subject: "transport_created",
          data: {
            type: data.type,
            options: {
              id: params.id,
              iceParameters: params.iceParameters,
              iceCandidates,
              dtlsParameters: params.dtlsParameters,
              sctpParameters: params.sctpParameters,
            },
          },
        });
        break;
      }

      case "produce": {
        players.produce(ws, data.rtpParameters);
        break;
      }

      case "produce_data": {
        if (data.sctpStreamParameters.streamId === undefined) {
          console.warn("produce_data: Stream ID is undefined");
          break;
        }

        players.produceData(ws, data.sctpStreamParameters as any);
        break;
      }

      case "set_rtp_capabilities": {
        players.setRtpCapabilities(ws, data.rtpCapabilities);
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
