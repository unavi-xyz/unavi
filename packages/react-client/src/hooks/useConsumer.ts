import { fromHostMessageSchema } from "@wired-labs/protocol";
import { Consumer } from "mediasoup-client/lib/Consumer";
import { Transport } from "mediasoup-client/lib/Transport";
import { useEffect, useState } from "react";

import { sendMessage } from "../utils/sendToHost";
import { useWebSocket } from "./useWebSocket";

/**
 * Manages WebRTC consumers.
 *
 * @param ws WebSocket connection
 * @param transport mediasoup-client transport
 * @param audioContext Audio context
 */
export function useConsumer(transport: Transport | null, audioContext: AudioContext) {
  const { ws } = useWebSocket();

  const [consumer, setConsumer] = useState<Consumer | null>(null);
  const [panners] = useState(new Map<number, PannerNode>());

  useEffect(() => {
    if (!ws || !transport) return;

    const onMessage = async (event: MessageEvent) => {
      const parsed = fromHostMessageSchema.safeParse(JSON.parse(event.data));

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
            if (audioContext.state === "suspended") audio.play();
            if (audioContext.state === "running") {
              document.removeEventListener("click", play);
              document.removeEventListener("touchstart", play);
            }
          };

          document.addEventListener("click", play);
          document.addEventListener("touchstart", play);

          // Create audio source
          const source = audioContext.createMediaStreamSource(audio.srcObject);

          // Create panner
          const panner = audioContext.createPanner();
          panner.panningModel = "HRTF";
          panner.rolloffFactor = 0.5;

          // Connect nodes
          source.connect(panner);
          panner.connect(audioContext.destination);

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
  }, [ws, transport, audioContext, panners]);

  useEffect(() => {
    if (!consumer) return;

    return () => {
      consumer.close();
    };
  }, [consumer]);

  return { consumer, panners };
}
