import { Triplet } from "@react-three/cannon";
import { useThree } from "@react-three/fiber";
import { useContext, useEffect, useRef } from "react";
import { Vector3 } from "three";

import { NetworkingContext } from "../networking";
import { LOCATION_PUBLISH_INTERVAL_MS } from "../networking/constants";
import { SentLocation } from "../networking/types";

export function usePublishLocation() {
  const tempVector3 = useRef(new Vector3());

  const { camera } = useThree();
  const { sendMessage } = useContext(NetworkingContext);

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
      const message: SentLocation = {
        type: "location",
        data: {
          position,
          rotation: angle,
        },
      };

      sendMessage(message);
    }, LOCATION_PUBLISH_INTERVAL_MS);

    return () => {
      clearInterval(interval);
    };
  }, [camera, sendMessage]);
}
