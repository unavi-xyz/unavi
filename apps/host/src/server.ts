import express from "express";
import { createServer } from "http";
import { RtpCapabilities } from "mediasoup/node/lib/RtpParameters";
import { Server } from "socket.io";

import { GameManager } from "./classes/GameManager";
import { createMediasoupRouter, createWebRtcTransport } from "./mediasoup";

const PORT = 4000;
const HOST = "0.0.0.0";

start();

async function start() {
  //create express server
  const app = express();

  //create http server
  const httpServer = createServer(app);

  //create socket server
  const io = new Server(httpServer, {
    cors: {
      origin: "*",
    },
  });

  //create mediasoup router
  const router = await createMediasoupRouter();

  //global variables
  const manager = new GameManager(router);

  //handle socket connections
  io.on("connection", (socket) => {
    const player = manager.createPlayer(socket);

    socket.on("join_space", async (data, callback) => {
      try {
        const spaceId = data.spaceId as string | undefined;
        if (!spaceId) throw new Error("spaceId is required");

        player.joinSpace(spaceId);

        callback({
          success: true,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("leave_space", async (data, callback) => {
      try {
        const spaceId = data.spaceId as string | undefined;
        if (!spaceId) throw new Error("spaceId is required");

        player.leaveSpace(spaceId);

        callback({
          success: true,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("get_router_rtp_capabilities", (callback) => {
      callback({
        success: true,
        routerRtpCapabilities: router.rtpCapabilities,
      });
    });

    socket.on("create_audio_producer_transport", async (callback) => {
      try {
        const { transport, params } = await createWebRtcTransport(router);
        player.audioProducer.transport = transport;

        callback({
          success: true,
          params,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("create_audio_consumer_transport", async (callback) => {
      try {
        const { transport, params } = await createWebRtcTransport(router);
        player.audioConsumer.transport = transport;

        callback({
          success: true,
          params,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("connect_audio_producer_transport", async (data, callback) => {
      try {
        if (!player.audioProducer.transport)
          throw new Error("audioProducerTransport is required");

        await player.audioProducer.transport.connect({
          dtlsParameters: data.dtlsParameters,
        });

        callback({
          success: true,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("connect_audio_consumer_transport", async (data, callback) => {
      try {
        if (!player.audioConsumer.transport)
          throw new Error("audioConsumerTransport is required");

        await player.audioConsumer.transport.connect({
          dtlsParameters: data.dtlsParameters,
        });

        callback({
          success: true,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("produce_audio", async (data, callback) => {
      try {
        const id = await player.produceAudio({
          kind: data.kind,
          rtpParameters: data.rtpParameters,
        });

        callback({
          success: true,
          id,
        });
      } catch (error) {
        console.error(error);
        callback({
          success: false,
        });
      }
    });

    socket.on("consume_audio", (data) => {
      const rtpCapabilities = data.rtpCapabilities as
        | RtpCapabilities
        | undefined;
      if (!rtpCapabilities) throw new Error("rtpCapabilities is required");

      player.rtpCapabilities = rtpCapabilities;
    });

    socket.on("disconnect", () => {
      //clean up player
      player.disconnect();
    });
  });

  //start http server
  httpServer.listen(PORT, HOST);
}
