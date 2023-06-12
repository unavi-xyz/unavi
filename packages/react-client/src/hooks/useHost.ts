import { RequestMessage, ResponseMessageSchema } from "@wired-protocol/types";
import { Device } from "mediasoup-client";
import { useContext, useEffect, useState } from "react";

import { ClientContext } from "../components/Client";
import { toHex } from "../utils/toHex";
import { usePlayers } from "./usePlayers";
import { usePublishData } from "./usePublishData";
import { useTransports } from "./useTransports";

/**
 * Hook to connect to a host.
 * This hook will create a WebSocket and WebRTC connection to the host and handle all
 * incoming and outgoing messages.
 */
export function useHost(uri: string | null, host: string | null) {
  const { engine, setWs, playerId, setPlayerId } = useContext(ClientContext);

  const [device, setDevice] = useState<Device | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [reconnectCount, setReconnectCount] = useState(0);

  const { dataProducer } = useTransports(device);
  usePlayers();
  usePublishData(dataProducer, playerId);

  useEffect(() => {
    if (!engine || !host || !uri) return;

    // Create WebSocket connection
    const hostURL =
      host.startsWith("ws://") || host.startsWith("wss://")
        ? host
        : host.startsWith("localhost")
        ? `ws://${host}`
        : `wss://${host}`;

    const newWs = new WebSocket(hostURL);
    setWs(newWs);

    // Create mediasoup device
    const newDevice = new Device();
    setDevice(newDevice);

    const send = (message: RequestMessage) => {
      if (!newWs || newWs.readyState !== newWs.OPEN) return;
      newWs.send(JSON.stringify(message));
    };

    newWs.onopen = () => {
      console.info("WebSocket - ✅ Connected to host");

      // Initiate WebRTC connection
      send({ data: null, id: "xyz.unavi.webrtc.router.rtpCapabilities.get" });

      // Join space
      send({ data: uri, id: "xyz.unavi.world.join" });

      engine.physics.addEventListener("user_grounded", (event) => {
        send({ data: event.data, id: "xyz.unavi.world.user.grounded" });
      });
    };

    newWs.onclose = () => {
      console.info("WebSocket - ❌ Connection closed");

      // Attempt to reconnect
      setReconnectCount((prev) => prev + 1);
    };

    newWs.onmessage = async (event: MessageEvent<string>) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { id, data } = parsed.data;

      switch (id) {
        case "xyz.unavi.world.joined": {
          console.info(`🌏 Joined space as player ${toHex(data)}`);

          setIsConnected(true);
          setPlayerId(data);
          break;
        }

        case "xyz.unavi.webrtc.router.rtpCapabilities": {
          if (newDevice.loaded) break;

          // Create device
          await newDevice.load({ routerRtpCapabilities: data });

          // Create transports
          send({ data: "producer", id: "xyz.unavi.webrtc.transport.create" });
          send({ data: "consumer", id: "xyz.unavi.webrtc.transport.create" });

          // Set rtp capabilities
          send({
            data: {
              codecs: newDevice.rtpCapabilities.codecs ?? [],
              headerExtensions: newDevice.rtpCapabilities.headerExtensions ?? [],
            },
            id: "xyz.unavi.webrtc.rtpCapabilities.set",
          });
          break;
        }
      }
    };

    // Try to play audio
    if (engine.audio.context.state === "suspended") engine.audio.context.resume();

    // Play audio on user interaction
    const play = () => {
      if (engine.audio.context.state === "suspended") engine.audio.context.resume();
      if (engine.audio.context.state === "running") {
        document.removeEventListener("click", play);
        document.removeEventListener("touchstart", play);
      }
    };

    document.addEventListener("click", play);
    document.addEventListener("touchstart", play);

    return () => {
      document.removeEventListener("click", play);
      document.removeEventListener("touchstart", play);

      newWs.close();

      setWs(null);
      setDevice(null);
      setPlayerId(null);
      setIsConnected(false);
      setReconnectCount(0);
    };
  }, [engine, setWs, setPlayerId, reconnectCount, host, uri]);

  return { isConnected };
}
