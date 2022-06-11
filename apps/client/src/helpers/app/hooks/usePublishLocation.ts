import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";

import { SpaceContext } from "../../../components/app/SpaceProvider";
import { PUBLISH_INTERVAL } from "../constants";
import { PlayerLocation } from "../types";

export function usePublishLocation() {
  const tempVector3 = useRef(new Vector3());

  const { camera } = useThree();
  const { sendLocation } = useContext(SpaceContext);

  useEffect(() => {
    //this is a hack
    //if the direction is exactly [0,0,-1] the camera faces the wrong direction
    camera.rotateOnAxis(new Vector3(0, 1, 0), 0.00000001);

    const interval = setInterval(() => {
      //get position
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      //get rotation
      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      //publish to space
      const location: PlayerLocation = { position, rotation: angle };
      sendLocation(location);
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [camera, sendLocation]);
}
