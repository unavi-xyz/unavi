import { useState } from "react";
import { Sky } from "@react-three/drei";
import { ASSET_NAMES, Player, Objects, Ground } from "3d";

import { EditorScene, useStore } from "../hooks/useStore";

function getSpawn(scene: EditorScene) {
  const object = Object.values(scene).find(
    (obj) => obj.instance.type === ASSET_NAMES.Spawn
  );

  if (!object) return;

  const spawn = object.instance.params.position;
  spawn[1] += 2;
  return spawn;
}

export default function PlayScene() {
  const scene = useStore((state) => state.scene);

  const objects = Object.values(scene).map((obj) => obj.instance);

  const [spawn] = useState(getSpawn(scene));

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Sky />
      <Ground />

      <Player spawn={spawn} />
      <Objects objects={objects} />
    </group>
  );
}
