import { useContext, useEffect, useRef, useState } from "react";
import { useThree } from "@react-three/fiber";
import { Triplet } from "@react-three/cannon";
import { Vector3 } from "three";
import { useAuth } from "ceramic";

import { PUBLISH_INTERVAL } from "../../helpers/constants";
import { MultiplayerContext } from "./MultiplayerContext";
import OtherPlayer from "./OtherPlayer";

interface Props {
  roomId: string;
}

export default function Multiplayer({ roomId }: Props) {
  const tempVector3 = useRef(new Vector3());

  const [players, setPlayers] = useState<string[]>([]);

  const { authenticated, viewerId } = useAuth();
  const { joinRoom, publishLocation, ydoc } = useContext(MultiplayerContext);

  const { camera } = useThree();

  useEffect(() => {
    if (!authenticated || !roomId) return;
    joinRoom(roomId);
  }, [authenticated, joinRoom, roomId]);

  useEffect(() => {
    const interval = setInterval(() => {
      const position: Triplet = camera.position.toArray();
      position[1] -= 1.5;

      const dir = camera.getWorldDirection(tempVector3.current);
      const sign = Math.sign(dir.x);
      const angle = Math.PI - (Math.atan(dir.z / dir.x) - (Math.PI / 2) * sign);

      publishLocation(position, angle);
    }, PUBLISH_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [camera, publishLocation]);

  useEffect(() => {
    const interval = setInterval(() => {
      const locations = ydoc.getMap("locations");
      const keys = [];

      locations.forEach((item, key) => {
        keys.push(key);
      });

      const newPlayer = Array.from(keys).find((key) => {
        if (players.includes(key) || key === viewerId) return false;
        return true;
      });

      if (newPlayer) setPlayers(Array.from(keys));
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [players, viewerId, ydoc]);

  return (
    <group>
      {players.map((did) => {
        if (did === viewerId) return null;
        return <OtherPlayer key={did} did={did} />;
      })}
    </group>
  );
}
