import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";

import { ConnectionContext } from "../../../components/app/ConnectionProvider";
import { PUBLISH_INTERVAL } from "../constants";
import { Location } from "../types";

export function usePublishLocation() {
  const tempVector3 = useRef(new Vector3());

  const { camera } = useThree();
  const { publishAll } = useContext(ConnectionContext);

  useEffect(() => {
    //this is a hack
    //if the direction is exactly [0,0,-1] the camera faces the wrong direction
    camera.rotateOnAxis(new Vector3(0, 1, 0), 0.00000001);

    const interval = setInterval(() => {
      //position
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      //rotation
      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      const location: Location = { position, rotation: angle };
      publishAll("location", location);
    }, PUBLISH_INTERVAL);

    return () => clearInterval(interval);
  }, [camera, publishAll]);
}
