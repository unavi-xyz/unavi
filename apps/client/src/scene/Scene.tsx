import { useRef } from "react";
import { Group } from "three";
import { Player } from "3d";

import World from "./World";

export default function Scene() {
  const world = useRef<undefined | Group>();

  return (
    <group>
      <Player world={world} />

      <group ref={world}>
        <World />
      </group>
    </group>
  );
}
