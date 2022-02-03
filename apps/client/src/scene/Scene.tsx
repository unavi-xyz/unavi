import { useRef } from "react";
import { Sky } from "@react-three/drei";
import { Group, Vector3 } from "three";
import { Player } from "3d";

import World from "./World";

export default function Scene() {
  const world = useRef<undefined | Group>();

  return (
    <group>
      <Player world={world} spawn={new Vector3(0, 2, 0)} />

      <group ref={world}>
        <Sky />
        <World />
      </group>
    </group>
  );
}
