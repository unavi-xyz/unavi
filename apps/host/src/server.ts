import {
  fromMediasoupRtpCapabilities,
  toMediasoupDtlsParameters,
  toMediasoupRtpCapabilities,
} from "@unavi/utils";
import {
  Request,
  RouterRtpCapabilities,
  TransportType,
} from "@wired-protocol/types";
import uWS from "uWebSockets.js";

import { createMediasoupWorker } from "./mediasoup";
import { Player } from "./Player";
import { UserData, uWebSocket } from "./types";
import { createTransport } from "./utils/createTransport";
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
    try {
      const player = players.get(ws);
      if (!player) return;

      if (buffer.byteLength === 0) {
        console.warn("Received empty message");
        return;
      }

      let req;

      try {
        const array = new Uint8Array(buffer);
        req = Request.fromBinary(array);
      } catch (err) {
        const text = Buffer.from(buffer).toString();
        console.log("Received text message", text);
        req = Request.fromJsonString(text);
      }

      switch (req.message.oneofKind) {
        case "join": {
          player.join(req.message.join.world);
          break;
        }

        case "leave": {
          player.leave(req.message.leave.world);
          break;
        }

        case "sendChatMessage": {
          player.chat(req.message.sendChatMessage.message);
          break;
        }

        case "setPlayerData": {
          for (const [key, value] of Object.entries(
            req.message.setPlayerData.data,
          )) {
            player.setPlayerData(key, value);
          }
          break;
        }

        case "sendEvent": {
          player.sendEvent(req.message.sendEvent.data);
          break;
        }

        case "getRouterRtpCapabilities": {
          const rtpCapabilities = fromMediasoupRtpCapabilities(
            router.rtpCapabilities,
          );
          const routerRtpCapabilities = RouterRtpCapabilities.create({
            rtpCapabilities,
          });
          player.send({
            oneofKind: "routerRtpCapabilities",
            routerRtpCapabilities,
          });
          break;
        }

        case "pauseAudio": {
          player.setPaused(req.message.pauseAudio.paused);
          break;
        }

        case "createTransport": {
          const type = req.message.createTransport.type;

          createTransport(type, router, webRtcServer)
            .then(({ transport, message }) => {
              player.setTransport(type, transport);
              player.send({
                oneofKind: "transportCreated",
                transportCreated: message,
              });
            })
            .catch((err) => console.warn(err));
          break;
        }

        case "connectTransport": {
          const transport =
            req.message.connectTransport.type === TransportType.PRODUCER
              ? player.producerTransport
              : player.consumerTransport;
          if (!transport || !req.message.connectTransport.dtlsParameters) break;

          transport.connect({
            dtlsParameters: toMediasoupDtlsParameters(
              req.message.connectTransport.dtlsParameters,
            ),
          });
          break;
        }

        case "produce": {
          if (!req.message.produce.rtpParameters) break;
          player.produce(req.message.produce.rtpParameters);
          break;
        }

        case "produceData": {
          if (!req.message.produceData.sctpStreamParameters) break;

          if (!req.message.produceData.sctpStreamParameters.streamId) {
            console.warn("Stream ID is undefined");
            break;
          }

          player.produceData({
            maxPacketLifeTime:
              req.message.produceData.sctpStreamParameters.maxPacketLifeTime,
            maxRetransmits:
              req.message.produceData.sctpStreamParameters.maxRetransmits,
            ordered: req.message.produceData.sctpStreamParameters.ordered,
            streamId: req.message.produceData.sctpStreamParameters.streamId,
          });
          break;
        }

        case "setRtpCapabilities": {
          if (!req.message.setRtpCapabilities.rtpCapabilities) break;

          const rtpCapabilities = toMediasoupRtpCapabilities(
            req.message.setRtpCapabilities.rtpCapabilities,
          );
          player.rtpCapabilities = rtpCapabilities;
          break;
        }
      }
    } catch (err) {
      console.warn(err);
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
