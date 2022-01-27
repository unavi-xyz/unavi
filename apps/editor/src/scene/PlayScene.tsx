import { useRef, useState } from "react";
import { Group, Vector3 } from "three";
import { Sky } from "@react-three/drei";
import { ASSET_NAMES, Ground, Player } from "3d";

import { IScene, useScene } from "../state/useScene";

import PlayObject from "./PlayObject";

function getSpawn(scene: IScene) {
  const object = Object.values(scene).find(
    (obj) => obj.params.type === ASSET_NAMES.Spawn
  );

  if (!object) return new Vector3(0, 2, 0);

  const spawn = new Vector3().fromArray(object.params.position);
  spawn.add(new Vector3(0, 2, 0));
  return spawn;
}

export default function PlayScene() {
  const world = useRef<undefined | Group>();

  const scene = useScene((state) => state.scene);

  const [spawn] = useState(getSpawn(scene));

  return (
    <group>
      <Player world={world} spawn={spawn} />

      <group ref={world}>
        <ambientLight intensity={0.1} />
        <directionalLight intensity={0.5} />
        <Sky />
        <Ground />

        {Object.values(scene).map((object) => {
          return <PlayObject key={object.id} object={object} />;
        })}
      </group>
    </group>
  );
}
