import { MessageSchema } from "@wired-labs/protocol";
import { Consumer } from "mediasoup-client/lib/Consumer";
import { Transport } from "mediasoup-client/lib/Transport";
import { useEffect, useState } from "react";

import { sendMessage } from "../utils/sendMessage";
import { useClient } from "./useClient";

/**
 * Manages WebRTC consumers.
 */
export function useConsumer(transport: Transport | null) {
  const { ws, engine } = useClient();

  const [consumer, setConsumer] = useState<Consumer | null>(null);
  const [panners] = useState(new Map<number, PannerNode>());

  useEffect(() => {
    if (!engine || !ws || !transport) return;

    const onMessage = async (event: MessageEvent) => {
      const parsed = MessageSchema.fromHost.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { subject, data } = parsed.data;

      switch (subject) {
        case "create_consumer": {
          const consumer = await transport.consume({
            id: data.consumerId,
            producerId: data.producerId,
            rtpParameters: data.rtpParameters,
            kind: "audio",
          });

          setConsumer(consumer);

          // Start receiving audio
          sendMessage(ws, { subject: "resume_audio", data: null });
          await consumer.resume();

          // Create audio stream
          const stream = new MediaStream([consumer.track.clone()]);

          // Create audio element
          const audio = new Audio();
          audio.srcObject = stream;
          audio.autoplay = true;

          // Play audio on user interaction
          const play = () => {
            if (engine.audio.context.state === "suspended") audio.play();
            if (engine.audio.context.state === "running") {
              document.removeEventListener("click", play);
              document.removeEventListener("touchstart", play);
            }
          };

          document.addEventListener("click", play);
          document.addEventListener("touchstart", play);

          // Create panner
          const panner = engine.audio.createAudioPanner();
          panner.rolloffFactor = 0.5;

          // Create audio source
          const source = engine.audio.context.createMediaStreamSource(audio.srcObject);
          source.connect(panner);

          // Store panner
          panners.set(data.playerId, panner);
          break;
        }
      }
    };

    ws.addEventListener("message", onMessage);

    return () => {
      ws.removeEventListener("message", onMessage);

      setConsumer(null);

      panners.forEach((panner) => {
        panner.disconnect();
      });
    };
  }, [ws, transport, engine, panners]);

  useEffect(() => {
    if (!consumer) return;

    return () => {
      consumer.close();
    };
  }, [consumer]);

  return { consumer, panners };
}
