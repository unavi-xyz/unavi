import { DataProducer } from "mediasoup-client/lib/DataProducer";
import { useEffect } from "react";

import { useClient } from "./useClient";

const PUBLISH_HZ = 15;

export function usePublishData(dataProducer: DataProducer | null, playerId: number | null) {
  const { engine } = useClient();

  useEffect(() => {
    if (!engine || !dataProducer || playerId === null) return;

    // Player id = 1 byte (Uint8)
    // Position = 3 * 4 bytes (Int32)
    // Rotation = 4 * 2 bytes (Int16)
    // Total = 21 bytes
    const bytes = 21;
    const buffer = new ArrayBuffer(bytes);
    const view = new DataView(buffer);

    const publishInterval = setInterval(() => {
      if (!engine.isPlaying || dataProducer.readyState !== "open") return;

      // Read location
      const posX = Atomics.load(engine.userPosition, 0);
      const posY = Atomics.load(engine.userPosition, 1);
      const posZ = Atomics.load(engine.userPosition, 2);

      const rotX = Atomics.load(engine.userRotation, 0);
      const rotY = Atomics.load(engine.userRotation, 1);
      const rotZ = Atomics.load(engine.userRotation, 2);
      const rotW = Atomics.load(engine.userRotation, 3);

      // Create buffer
      view.setUint8(0, playerId);

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

    return () => {
      clearInterval(publishInterval);
    };
  }, [dataProducer, engine, playerId]);
}
