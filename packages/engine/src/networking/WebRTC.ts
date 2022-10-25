import { Device } from "mediasoup-client";
import { Producer } from "mediasoup-client/lib/Producer";
import { Transport } from "mediasoup-client/lib/Transport";

import { RenderThread } from "../render/RenderThread";
import { quaternionToYaw } from "../render/utils/quaternionToYaw";
import { FromHostMessage, ToHostMessage } from "./types";

const PUBLISH_HZ = 15; // X times per second

export class WebRTC {
  playerId: number | null = null;
  playerPosition: Int32Array | null = null;
  playerRotation: Int16Array | null = null;

  #ws: WebSocket;
  #renderThread: RenderThread;

  #audioContext = new AudioContext();
  #device: Device | null = null;
  #producerTransport: Transport | null = null;
  #consumerTransport: Transport | null = null;
  #producer: Producer | null = null;
  #paused = false;
  #producedTrack: MediaStreamTrack | null = null;

  #panners = new Map<number, PannerNode>();

  #onProducerId: ({ id }: { id: string }) => void = () => {};
  #onDataProducerId: ({ id }: { id: string }) => void = () => {};

  constructor(
    ws: WebSocket,
    renderThread: RenderThread,
    producedTrack?: MediaStreamTrack | null
  ) {
    this.#ws = ws;
    this.#renderThread = renderThread;
    if (producedTrack) this.#producedTrack = producedTrack;
  }

  connect() {
    this.#send({ subject: "get_router_rtp_capabilities", data: null });
  }

  async onmessage({ subject, data }: FromHostMessage) {
    switch (subject) {
      case "router_rtp_capabilities": {
        // Create device
        const device = new Device();
        await device.load({ routerRtpCapabilities: data });
        this.#device = device;

        // Create transports
        this.#send({
          subject: "create_transport",
          data: { type: "producer" },
        });

        this.#send({
          subject: "create_transport",
          data: { type: "consumer" },
        });

        // Set rtp capabilities
        this.#send({
          subject: "set_rtp_capabilities",
          data: {
            rtpCapabilities: this.#device.rtpCapabilities,
          },
        });
        break;
      }

      case "transport_created": {
        if (!this.#device) throw new Error("Device not initialized");

        // Create local transport
        const transport =
          data.type === "producer"
            ? this.#device.createSendTransport(data.options)
            : this.#device.createRecvTransport(data.options);

        transport.on("connect", async ({ dtlsParameters }, callback) => {
          this.#send({
            subject: "connect_transport",
            data: {
              dtlsParameters,
              type: data.type,
            },
          });

          callback();
        });

        transport.on("connectionstatechange", (state) => {
          console.info(`🚀 WebRTC ${data.type}: ${state}`);

          if (state === "connected" && data.type === "producer") {
            if (this.#producedTrack) this.produceAudio(this.#producedTrack);
          }
        });

        if (data.type === "producer") {
          this.#producerTransport = transport;

          transport.on("produce", async ({ kind, rtpParameters }, callback) => {
            if (kind === "video") throw new Error("Video not supported");

            this.#onProducerId = callback;

            this.#send({
              subject: "produce",
              data: { rtpParameters },
            });
          });

          transport.on(
            "producedata",
            async ({ sctpStreamParameters }, callback) => {
              this.#onDataProducerId = callback;

              this.#send({
                subject: "produce_data",
                data: { sctpStreamParameters },
              });
            }
          );

          const dataProducer = await transport.produceData({
            ordered: false,
            maxRetransmits: 0,
          });

          // Player id = 1 byte (Uint8)
          // Position = 3 * 4 bytes (Int32)
          // Rotation = 4 * 2 bytes (Int16)
          // Total = 21 bytes
          const bytes = 21;
          const buffer = new ArrayBuffer(bytes);
          const view = new DataView(buffer);

          setInterval(() => {
            if (
              this.playerId === null ||
              !this.playerPosition ||
              !this.playerRotation ||
              dataProducer.readyState !== "open"
            )
              return;

            // Read location
            const posX = Atomics.load(this.playerPosition, 0);
            const posY = Atomics.load(this.playerPosition, 1);
            const posZ = Atomics.load(this.playerPosition, 2);

            const rotX = Atomics.load(this.playerRotation, 0);
            const rotY = Atomics.load(this.playerRotation, 1);
            const rotZ = Atomics.load(this.playerRotation, 2);
            const rotW = Atomics.load(this.playerRotation, 3);

            // Set audio listener position
            const listener = this.#audioContext.listener;

            if (listener.positionX !== undefined) {
              listener.positionX.value = posX / 1000;
              listener.positionY.value = posY / 1000;
              listener.positionZ.value = posZ / 1000;
            } else {
              listener.setPosition(posX / 1000, posY / 1000, posZ / 1000);
            }

            const yaw = quaternionToYaw(rotY / 1000, rotW / 1000);

            if (listener.forwardX !== undefined) {
              listener.forwardX.value = -Math.sin(yaw);
              listener.forwardZ.value = -Math.cos(yaw);
            } else {
              listener.setOrientation(
                -Math.sin(yaw),
                0,
                -Math.cos(yaw),
                0,
                1,
                0
              );
            }

            // Create buffer
            view.setUint8(0, this.playerId);

            view.setInt32(1, posX, true);
            view.setInt32(5, posY, true);
            view.setInt32(9, posZ, true);

            view.setInt16(13, rotX, true);
            view.setInt16(15, rotY, true);
            view.setInt16(17, rotZ, true);
            view.setInt16(19, rotW, true);

            // Publish buffer
            dataProducer.send(buffer);
          }, 1000 / PUBLISH_HZ);
        }

        if (data.type === "consumer") {
          this.#consumerTransport = transport;

          this.#send({
            subject: "ready_to_consume",
            data: true,
          });
        }

        break;
      }

      case "create_consumer": {
        if (!this.#consumerTransport)
          throw new Error("Consumer transport not initialized");
        if (this.#consumerTransport.closed)
          throw new Error("Consumer transport closed");

        const consumer = await this.#consumerTransport.consume({
          id: data.id,
          producerId: data.producerId,
          rtpParameters: data.rtpParameters,
          kind: "audio",
        });

        // Start receiving audio
        this.#send({
          subject: "resume_audio",
          data: null,
        });

        await consumer.resume();

        // Create audio stream
        const stream = new MediaStream([consumer.track.clone()]);

        // Create audio element
        const audio = new Audio();
        audio.srcObject = stream;

        // Create audio source
        const source = this.#audioContext.createMediaStreamSource(
          audio.srcObject
        );

        // Create panner
        const panner = this.#audioContext.createPanner();
        panner.panningModel = "HRTF";
        panner.rolloffFactor = 0.5;

        // Connect nodes
        source.connect(panner);
        panner.connect(this.#audioContext.destination);

        if (this.#audioContext.state === "suspended")
          this.#audioContext.resume();

        // Play audio on user interaction
        const play = async () => {
          if (this.#audioContext.state === "suspended")
            await this.#audioContext.resume();
          document.removeEventListener("click", play);
        };

        document.addEventListener("click", play);

        // Store panner
        this.#panners.set(data.playerId, panner);
        break;
      }

      case "create_data_consumer": {
        if (!this.#consumerTransport)
          throw new Error("Consumer transport not initialized");
        if (this.#consumerTransport.closed)
          throw new Error("Consumer transport closed");

        const dataConsumer = await this.#consumerTransport.consumeData({
          id: data.id,
          dataProducerId: data.dataProducerId,
          sctpStreamParameters: data.sctpStreamParameters,
        });

        dataConsumer.on("message", (buffer) => {
          // Apply location to audio panner
          const view = new DataView(buffer);
          const playerId = view.getUint8(0);

          const panner = this.#panners.get(playerId);
          if (panner) {
            if (panner.positionX !== undefined) {
              panner.positionX.value = view.getInt32(1, true) / 1000;
              panner.positionY.value = view.getInt32(5, true) / 1000;
              panner.positionZ.value = view.getInt32(9, true) / 1000;
            } else {
              panner.setPosition(
                view.getInt32(1, true) / 1000,
                view.getInt32(5, true) / 1000,
                view.getInt32(9, true) / 1000
              );
            }
          }

          // Send to renderThread
          this.#renderThread.postMessage(
            {
              subject: "player_location",
              data: buffer,
            },
            [buffer]
          );
        });

        break;
      }

      case "producer_id": {
        this.#onProducerId(data);
        break;
      }

      case "data_producer_id": {
        this.#onDataProducerId(data);
        break;
      }
    }
  }

  #send(message: ToHostMessage) {
    this.#ws.send(JSON.stringify(message));
  }

  async produceAudio(track: MediaStreamTrack) {
    if (!this.#producerTransport)
      throw new Error("Producer transport not initialized");
    if (this.#producerTransport.closed)
      throw new Error("Producer transport closed");

    this.#producer = await this.#producerTransport.produce({ track });

    if (this.#paused) this.#producer.pause();
    else this.#producer.resume();

    this.#producedTrack = track;
  }

  setAudioPaused(paused: boolean) {
    this.#paused = paused;

    if (this.#producer && paused) this.#producer.pause();
    else if (this.#producer && !paused) this.#producer.resume();
  }
}
