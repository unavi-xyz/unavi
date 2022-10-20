import { Device } from "mediasoup-client";
import { Transport } from "mediasoup-client/lib/Transport";

import { FromHostMessage, ToHostMessage } from "./types";

const PUBLISH_HZ = 15; // X times per second

export class WebRTC {
  #ws: WebSocket;

  #device: Device | null = null;

  #consumerTransport: Transport | null = null;

  constructor(ws: WebSocket) {
    this.#ws = ws;
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
          data: {
            type: "producer",
          },
        });

        this.#send({
          subject: "create_transport",
          data: {
            type: "consumer",
          },
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
        });

        if (data.type === "producer") {
          transport.on("produce", async ({ kind, rtpParameters }) => {
            if (kind === "video") throw new Error("Video not supported");

            this.#send({
              subject: "produce",
              data: { rtpParameters },
            });
          });

          transport.on("producedata", async ({ sctpStreamParameters }) => {
            this.#send({
              subject: "produce_data",
              data: { sctpStreamParameters },
            });
          });

          const dataProducer = await transport.produceData({
            ordered: false,
            maxRetransmits: 0,
          });

          setInterval(() => {
            dataProducer.send("ping");
          }, 1000);
        } else {
          this.#consumerTransport = transport;
        }

        break;
      }

      case "create_consumer": {
        if (!this.#consumerTransport)
          throw new Error("Consumer transport not initialized");

        const consumer = await this.#consumerTransport.consume({
          producerId: data.producerId,
          rtpParameters: data.rtpParameters,
        });

        break;
      }

      case "create_data_consumer": {
        if (!this.#consumerTransport)
          throw new Error("Consumer transport not initialized");

        const dataConsumer = await this.#consumerTransport.consumeData({
          id: data.id,
          dataProducerId: data.dataProducerId,
          sctpStreamParameters: data.sctpStreamParameters,
        });

        break;
      }
    }
  }

  #send(message: ToHostMessage) {
    this.#ws.send(JSON.stringify(message));
  }
}
