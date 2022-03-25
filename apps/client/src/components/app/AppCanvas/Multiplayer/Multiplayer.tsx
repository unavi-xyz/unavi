import { useContext, useEffect, useRef, useState } from "react";
import { useThree } from "@react-three/fiber";
import { Triplet } from "@react-three/cannon";
import { Vector3 } from "three";

import { PUBLISH_INTERVAL } from "./constants";
import { YLocation } from "../../helpers/types";
import { MultiplayerContext } from "../../MultiplayerProvider";

import OtherPlayer from "./OtherPlayer";

export default function Multiplayer() {
  const tempVector3 = useRef(new Vector3());

  const [players, setPlayers] = useState<string[]>([]);

  const { publishLocation, ydoc, username, getLocations } =
    useContext(MultiplayerContext);

  const { camera } = useThree();

  useEffect(() => {
    const interval = setInterval(() => {
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      const location: YLocation = { position, rotation: angle };
      publishLocation(location);
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [camera, publishLocation]);

  useEffect(() => {
    const interval = setInterval(() => {
      const locations = getLocations();

      const newPlayer = Object.keys(locations).find((key) => {
        if (players.includes(key) || key === username) return false;
        return true;
      });

      if (newPlayer) setPlayers(Object.keys(locations));
    }, 1000);

    return () => {
      clearInterval(interval);
    };
  }, [getLocations, players, username]);

  return (
    <group>
      {players.map((id) => {
        if (id === username) return null;
        return <OtherPlayer key={id} id={id} />;
      })}
    </group>
  );
}
