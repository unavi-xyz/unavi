import { ResponseMessageSchema } from "@wired-protocol/types";
import { Producer, Transport } from "mediasoup-client/lib/types";
import { useEffect, useState } from "react";

import { sendMessage } from "../utils/sendMessage";
import { useClient } from "./useClient";

/**
 * Manages a mediasoup producer transport.
 *
 * @param transport Producer transport
 * @param playerId Player ID
 * @returns Producer
 */
export function useProducer(transport: Transport | null) {
  const { ws, micEnabled, micTrack } = useClient();

  const [producer, setProducer] = useState<Producer | null>(null);
  const [producerIdCallback, setProducerIdCallback] = useState<((id: string) => void) | null>(null);
  const [dataProducerIdCallback, setDataProducerIdCallback] = useState<
    ((id: string) => void) | null
  >(null);
  const [transportConnected, setTransportConnected] = useState(false);

  useEffect(() => {
    if (!ws || !transport) return;

    transport.on("connectionstatechange", async (state) => {
      if (state === "connected") setTransportConnected(true);
    });

    transport.on("produce", async ({ kind, rtpParameters }, callback) => {
      if (kind === "video") throw new Error("Video not supported");
      setProducerIdCallback((id: string) => callback({ id }));
      sendMessage(ws, { type: "webrtc_produce", data: rtpParameters });
    });

    transport.on("producedata", async ({ sctpStreamParameters }, callback) => {
      setDataProducerIdCallback((id: string) => callback({ id }));
      sendMessage(ws, { type: "webrtc_produce_data", data: sctpStreamParameters });
    });

    return () => {
      transport.removeAllListeners();
      setProducer(null);
    };
  }, [ws, transport]);

  useEffect(() => {
    if (!ws || !transport) return;

    const onMessage = (event: MessageEvent) => {
      const parsed = ResponseMessageSchema.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { type, data } = parsed.data;

      switch (type) {
        case "webrtc_producer_id": {
          if (producerIdCallback) producerIdCallback(data);
          break;
        }

        case "webrtc_data_producer_id": {
          if (dataProducerIdCallback) dataProducerIdCallback(data);
          break;
        }
      }
    };

    ws.addEventListener("message", onMessage);

    return () => {
      ws.removeEventListener("message", onMessage);

      setTransportConnected(false);
      setProducerIdCallback(null);
      setDataProducerIdCallback(null);
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

  useEffect(() => {
    if (!transport || !transportConnected || !micTrack) return;

    transport.produce({ track: micTrack }).then((newProducer) => {
      setProducer(newProducer);
    });
  }, [transportConnected, micTrack, transport]);

  return producer;
}
