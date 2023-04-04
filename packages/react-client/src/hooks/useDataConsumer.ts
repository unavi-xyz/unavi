import { POSITION_ARRAY_ROUNDING, quaternionToEuler, ROTATION_ARRAY_ROUNDING } from "engine";
import { DataConsumer } from "mediasoup-client/lib/DataConsumer";
import { useEffect } from "react";

import { useClient } from "./useClient";

export function useDataConsumer(
  dataConsumer: DataConsumer | null,
  panners: Map<number, PannerNode>
) {
  const { engine } = useClient();

  useEffect(() => {
    if (!dataConsumer) return;

    return () => {
      dataConsumer.close();
    };
  }, [dataConsumer]);

  useEffect(() => {
    if (!dataConsumer || !engine) return;

    const onMessage = async (data: ArrayBuffer | Blob) => {
      const buffer = data instanceof ArrayBuffer ? data : await data.arrayBuffer();

      const view = new DataView(buffer);

      const playerId = view.getUint8(0);

      const posX = view.getInt32(1, true) / POSITION_ARRAY_ROUNDING;
      const posY = view.getInt32(5, true) / POSITION_ARRAY_ROUNDING;
      const posZ = view.getInt32(9, true) / POSITION_ARRAY_ROUNDING;

      const rotX = view.getInt16(13, true) / ROTATION_ARRAY_ROUNDING;
      const rotY = view.getInt16(15, true) / ROTATION_ARRAY_ROUNDING;
      const rotZ = view.getInt16(17, true) / ROTATION_ARRAY_ROUNDING;
      const rotW = view.getInt16(19, true) / ROTATION_ARRAY_ROUNDING;

      // Apply location to audio panner
      const panner = panners.get(playerId);

      if (panner) {
        if (panner.positionX !== undefined) {
          panner.positionX.value = posX;
          panner.positionY.value = posY;
          panner.positionZ.value = posZ;
        } else {
          panner.setPosition(posX, posY, posZ);
        }

        const [eulX, eulY, eulZ] = quaternionToEuler([rotX, rotY, rotZ, rotW]);

        if (panner.orientationX !== undefined) {
          panner.orientationX.value = eulX;
          panner.orientationY.value = eulY;
          panner.orientationZ.value = eulZ;
        } else {
          panner.setOrientation(eulX, eulY, eulZ);
        }
      }

      // Send to engine
      const player = engine.player.getPlayer(playerId);

      if (player) {
        player.setPosition(posX, posY, posZ);
        player.setRotation(rotX, rotY, rotZ, rotW);
      }
    };

    dataConsumer.on("message", onMessage);

    return () => {
      dataConsumer.off("message", onMessage);
    };
  }, [dataConsumer, engine, panners]);
}
