import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";

import { SpaceContext } from "../../../components/app/SpaceProvider";
import { PUBLISH_INTERVAL } from "../constants";
import { Location } from "../types";

export function usePublishLocation() {
  const tempVector3 = useRef(new Vector3());

  const { camera } = useThree();
  const { publishAll } = useContext(SpaceContext);

  useEffect(() => {
    const interval = setInterval(() => {
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      const location: Location = { position, rotation: angle };
      publishAll("location", location);
    }, PUBLISH_INTERVAL);

    return () => clearInterval(interval);
  }, [camera, publishAll]);
}
