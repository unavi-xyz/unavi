import {
  ConnectTrannsport,
  CreateTransport,
  Join,
  Leave,
  Message,
  PauseAudio,
  Produce,
  ProduceData,
  RouterRtpCapabilities,
  SendChatMessage,
  SetAvatar,
  SetFalling,
  SetHandle,
  SetNickname,
  SetRtpCapabilities,
  TransportCreated,
  TransportType,
} from "@wired-protocol/types";
import uWS from "uWebSockets.js";

import { REQ_WEBRTC, REQ_WORLD, RES_WEBRTC } from "./constants";
import { createMediasoupWorker } from "./mediasoup";
import { Player } from "./Player";
import { UserData, uWebSocket } from "./types";
import { createTransport } from "./utils/createTransport";
import { createMediasoupRtpCapabilities } from "./utils/rtpCapabilities";
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

    const array = new Uint8Array(buffer);
    const msg = Message.fromBinary(array);

    switch (msg.type) {
      case `${REQ_WORLD}.Join`: {
        const data = Join.fromBinary(msg.data);
        player.join(data.world);
        break;
      }

      case `${REQ_WORLD}.Leave`: {
        const data = Leave.fromBinary(msg.data);
        player.leave(data.world);
        break;
      }

      case `${REQ_WORLD}.SendChatMessage`: {
        const data = SendChatMessage.fromBinary(msg.data);
        player.chat(data.message);
        break;
      }

      case `${REQ_WORLD}.SetFalling`: {
        const data = SetFalling.fromBinary(msg.data);
        player.falling = data.falling;
        break;
      }

      case `${REQ_WORLD}.SetNickname`: {
        const data = SetNickname.fromBinary(msg.data);
        player.name = data.nickname;
        break;
      }

      case `${REQ_WORLD}.SetAvatar`: {
        const data = SetAvatar.fromBinary(msg.data);
        player.avatar = data.avatar;
        break;
      }

      case `${REQ_WORLD}.SetHandle`: {
        const data = SetHandle.fromBinary(msg.data);
        player.handle = data.handle;
        break;
      }

      case `${REQ_WEBRTC}.GetRouterRtpCapabilities`: {
        const rtpCapabilities = createRouterRtpCapabilities(router);
        player.send({
          data: RouterRtpCapabilities.toBinary(rtpCapabilities),
          type: `${REQ_WEBRTC}.RouterRtpCapabilities`,
        });
        break;
      }

      case `${REQ_WEBRTC}.PauseAudio`: {
        const data = PauseAudio.fromBinary(msg.data);
        player.setPaused(data.paused);
        break;
      }

      case `${REQ_WEBRTC}.CreateTransport`: {
        const data = CreateTransport.fromBinary(msg.data);

        createTransport(data.type, router, webRtcServer)
          .then(({ transport, message }) => {
            player.setTransport(data.type, transport);
            player.send({
              data: TransportCreated.toBinary(message),
              type: `${RES_WEBRTC}.TransportCreated`,
            });
          })
          .catch((err) => console.warn(err));
        break;
      }

      case `${REQ_WEBRTC}.ConnectTransport`: {
        const data = ConnectTrannsport.fromBinary(msg.data);

        const transport =
          data.type === TransportType.PRODUCER
            ? player.producerTransport
            : player.consumerTransport;
        if (!transport) break;

        transport.connect({ dtlsParameters: data.dtlsParameters });
        break;
      }

      case `${REQ_WEBRTC}.Produce`: {
        const data = Produce.fromBinary(msg.data);
        if (!data.rtpParameters) break;

        player.produce(data.rtpParameters);
        break;
      }

      case "com.wired-protocol.webrtc.produceData": {
        const data = ProduceData.fromBinary(msg.data);
        if (!data.sctpStreamParameters) break;

        if (data.sctpStreamParameters.streamId === undefined) {
          console.warn("Stream ID is undefined");
          break;
        }

        player.produceData({
          maxPacketLifeTime: data.sctpStreamParameters.maxPacketLifeTime,
          maxRetransmits: data.sctpStreamParameters.maxRetransmits,
          ordered: data.sctpStreamParameters.ordered,
          streamId: data.sctpStreamParameters.streamId,
        });
        break;
      }

      case `${REQ_WEBRTC}.SetRtpCapabilities`: {
        const data = SetRtpCapabilities.fromBinary(msg.data);
        if (!data.rtpCapabilities) break;

        const rtpCapabilities = createMediasoupRtpCapabilities(
          data.rtpCapabilities,
        );
        player.rtpCapabilities = rtpCapabilities;
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
