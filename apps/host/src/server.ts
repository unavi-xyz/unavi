import express from "express";
import { createServer } from "http";
import { Server } from "socket.io";

import {
  ConnectTransportDataSchema,
  ConsumeAudioDataSchema,
  JoinSpaceDataSchema,
  LeaveSpaceDataSchema,
  ProduceAudioDataSchema,
} from "@wired-xr/engine/src/networking/schemas";

import { GameManager } from "./classes/GameManager";
import { createMediasoupRouter, createWebRtcTransport } from "./mediasoup";
import { TypedSocket } from "./types";

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
  io.on("connection", (socket: TypedSocket) => {
    const player = manager.createPlayer(socket);

    socket.on("join_space", async (data, callback) => {
      try {
        const { spaceId } = JoinSpaceDataSchema.parse(data);

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
        const { spaceId } = LeaveSpaceDataSchema.parse(data);

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
        const { dtlsParameters } = ConnectTransportDataSchema.parse(data);

        if (!player.audioProducer.transport)
          throw new Error("audioProducerTransport is required");

        await player.audioProducer.transport.connect({
          dtlsParameters,
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
        const { kind, rtpParameters } = ProduceAudioDataSchema.parse(data);

        const id = await player.produceAudio({
          kind,
          rtpParameters,
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

    socket.on("consume_audio", (data, callback) => {
      try {
        const { rtpCapabilities } = ConsumeAudioDataSchema.parse(data);

        player.rtpCapabilities = rtpCapabilities;

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

    socket.on("disconnect", () => {
      //clean up player
      player.disconnect();
    });
  });

  //start http server
  httpServer.listen(PORT, HOST);
}
