import { useEffect, useRef } from "react";
import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { Vector3 } from "three";

import { appManager } from "../../../helpers/store";
import { Transform } from "../../../helpers/types";
import { PUBLISH_INTERVAL } from "../helpers/constants";

export default function usePublishPosition() {
  const tempVector3 = useRef(new Vector3());

  const { camera } = useThree();

  useEffect(() => {
    const interval = setInterval(() => {
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      const transform: Transform = { position, rotation: angle };
      appManager.publishAll("transform", transform);
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [camera]);
}
