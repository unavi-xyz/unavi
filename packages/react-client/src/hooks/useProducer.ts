import { fromHostMessageSchema } from "@wired-labs/protocol";
import { Producer } from "mediasoup-client/lib/Producer";
import { Transport } from "mediasoup-client/lib/Transport";
import { useEffect, useState } from "react";

import { sendMessage } from "../utils/sendToHost";
import { useMic } from "./useMic";
import { useWebSocket } from "./useWebSocket";

/**
 * Manages a mediasoup producer transport.
 *
 * @param transport Producer transport
 * @param playerId Player ID
 * @returns Producer
 */
export function useProducer(transport: Transport | null) {
  const { ws } = useWebSocket();
  const { micEnabled } = useMic();

  const [producer, setProducer] = useState<Producer | null>(null);
  const [producerIdCallback, setProducerIdCallback] = useState<((id: string) => void) | null>(null);
  const [dataProducerIdCallback, setDataProducerIdCallback] = useState<
    ((id: string) => void) | null
  >(null);

  useEffect(() => {
    if (!ws || !transport) return;

    transport.on("connectionstatechange", async (state) => {
      if (state === "connected") {
        // TODO Produce audio on connect - get produced track from user
        // if (producedTrack) {
        //   const newProducer = await transport.produce({ track: producedTrack });
        //   setProducer(newProducer);
        // }
      }
    });

    transport.on("produce", async ({ kind, rtpParameters }, callback) => {
      if (kind === "video") throw new Error("Video not supported");
      setProducerIdCallback((id: string) => callback({ id }));
      sendMessage(ws, { subject: "produce", data: rtpParameters });
    });

    transport.on("producedata", async ({ sctpStreamParameters }, callback) => {
      setDataProducerIdCallback((id: string) => callback({ id }));
      sendMessage(ws, { subject: "produce_data", data: sctpStreamParameters });
    });

    return () => {
      transport.removeAllListeners();
      setProducer(null);
    };
  }, [ws, transport]);

  useEffect(() => {
    if (!ws || !transport) return;

    const onMessage = (event: MessageEvent) => {
      const parsed = fromHostMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { subject, data } = parsed.data;

      switch (subject) {
        case "producer_id": {
          if (producerIdCallback) producerIdCallback(data);
          break;
        }

        case "data_producer_id": {
          if (dataProducerIdCallback) dataProducerIdCallback(data);
          break;
        }
      }
    };

    ws.addEventListener("message", onMessage);

    return () => {
      ws.removeEventListener("message", onMessage);
    };
  }, [ws, transport, producerIdCallback, dataProducerIdCallback]);

  useEffect(() => {
    if (!producer) return;

    // Publish mic audio
    if (micEnabled) producer.resume();
    else producer.pause();
  }, [producer, micEnabled]);

  useEffect(() => {
    if (!producer) return;

    return () => {
      producer.close();
    };
  }, [producer]);

  return producer;
}
