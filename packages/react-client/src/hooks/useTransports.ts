import { MessageSchema } from "@wired-labs/protocol";
import { Device } from "mediasoup-client";
import { DataConsumer } from "mediasoup-client/lib/DataConsumer";
import { Transport } from "mediasoup-client/lib/Transport";
import { useEffect, useState } from "react";

import { sendMessage } from "../utils/sendMessage";
import { useClient } from "./useClient";
import { useConsumer } from "./useConsumer";
import { useDataConsumer } from "./useDataConsumer";
import { useDataProducer } from "./useDataProducer";
import { useProducer } from "./useProducer";

/**
 * Manages WebRTC transports.
 */
export function useTransports(device: Device | null) {
  const { ws } = useClient();

  const [consumerTransport, setConsumerTransport] = useState<Transport | null>(null);
  const [producerTransport, setProducerTransport] = useState<Transport | null>(null);
  const [dataConsumer, setDataConsumer] = useState<DataConsumer | null>(null);

  const producer = useProducer(producerTransport);
  const dataProducer = useDataProducer(producerTransport);

  const { panners } = useConsumer(consumerTransport);
  useDataConsumer(dataConsumer, panners);

  useEffect(() => {
    if (!ws || !device) return;

    let localConsumerTransport: Transport | null = null;

    const onMessage = async (event: MessageEvent) => {
      const parsed = MessageSchema.fromHost.safeParse(JSON.parse(event.data));

      if (!parsed.success) {
        console.warn(parsed.error);
        return;
      }

      const { subject, data } = parsed.data;

      switch (subject) {
        case "transport_created": {
          // Create local transport
          const transport =
            data.type === "producer"
              ? device.createSendTransport(data.options as any)
              : device.createRecvTransport(data.options as any);

          if (data.type === "consumer") {
            localConsumerTransport = transport;
            setConsumerTransport(transport);
          } else {
            setProducerTransport(transport);
          }

          transport.on("connect", ({ dtlsParameters }, callback) => {
            sendMessage(ws, {
              subject: "connect_transport",
              data: { dtlsParameters, type: data.type },
            });
            callback();
          });

          transport.on("connectionstatechange", (state) => {
            console.info(`WebRTC - ${data.type} ${state}`);
          });
          break;
        }

        case "create_data_consumer": {
          if (!localConsumerTransport) throw new Error("No consumer transport");

          const newDataConsumer = await localConsumerTransport.consumeData({
            id: data.consumerId,
            dataProducerId: data.dataProducerId,
            sctpStreamParameters: data.sctpStreamParameters,
          });

          setDataConsumer(newDataConsumer);
          break;
        }
      }
    };

    ws.addEventListener("message", onMessage);

    return () => {
      ws.removeEventListener("message", onMessage);
    };
  }, [ws, device]);

  return { consumerTransport, producerTransport, producer, dataProducer };
}
