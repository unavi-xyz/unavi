import { RequestMessage, ResponseMessageSchema } from "@wired-protocol/types";
import { Device } from "mediasoup-client";
import { Query, SystemRes } from "thyseus";

import { WorldJson } from "../components";
import { useClientStore } from "../store";
import { toHex } from "../utils/toHex";

class LocalRes {
  host = "";
  ws: WebSocket | null = null;
}

export function connectToHost(
  localRes: SystemRes<LocalRes>,
  worlds: Query<WorldJson>
) {
  for (const world of worlds) {
    if (localRes.host === world.host) continue;

    localRes.host = world.host;
    if (localRes.ws) localRes.ws.close();

    const prefix = world.host.startsWith("localhost") ? "ws://" : "wss://";
    const ws = new WebSocket(`${prefix}${world.host}`);
    localRes.ws = ws;

    const sendQueue: RequestMessage[] = [];

    const send = (message: RequestMessage) => {
      if (ws.readyState === ws.OPEN) {
        ws.send(JSON.stringify(message));
      } else {
        sendQueue.push(message);
      }
    };

    ws.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Send all queued messages
      for (const message of sendQueue) {
        send(message);
      }
      sendQueue.length = 0;

      // Initiate WebRTC connection
      send({ data: null, id: "xyz.unavi.webrtc.router.rtpCapabilities.get" });

      // Join world
      const uri = useClientStore.getState().worldUri;
      send({ data: uri, id: "xyz.unavi.world.join" });
    };

    ws.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");
    };

    ws.onerror = (error) => {
      console.error("WebSocket - ⚠️ Connection error", error);
    };

    ws.onmessage = async (event) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { data, id } = parsed.data;

      switch (id) {
        case "xyz.unavi.world.joined": {
          console.info(`🌏 Joined world as player ${toHex(data)}`);
          break;
        }

        case "xyz.unavi.webrtc.router.rtpCapabilities": {
          // Create mediasoup device
          const device = new Device();

          try {
            await device.load({ routerRtpCapabilities: data });

            // Create transports
            send({
              data: "producer",
              id: "xyz.unavi.webrtc.transport.create",
            });
            send({
              data: "consumer",
              id: "xyz.unavi.webrtc.transport.create",
            });

            // Set rtp capabilities
            send({
              data: {
                codecs: device.rtpCapabilities.codecs ?? [],
                headerExtensions: device.rtpCapabilities.headerExtensions ?? [],
              },
              id: "xyz.unavi.webrtc.rtpCapabilities.set",
            });
          } catch (error) {
            console.error("Error loading device", error);
          }

          break;
        }
      }
    };
  }
}
