import { useRef, useState } from "react";
import { Group } from "three";
import { ASSET_NAMES, Player, Scene } from "3d";

import { EditorScene, useScene } from "../state/useScene";

function getSpawn(scene: EditorScene) {
  const object = Object.values(scene).find(
    (obj) => obj.params.type === ASSET_NAMES.Spawn
  );

  if (!object) return;

  const spawn = object.params.position;
  spawn[1] += 2;
  return spawn;
}

export default function PlayScene() {
  const world = useRef<undefined | Group>();

  const scene = useScene((state) => state.scene);

  const [spawn] = useState(getSpawn(scene));

  const objects = Object.values(scene).map((obj) => obj.params);

  return (
    <group>
      <Player world={world} spawn={spawn} />
      <Scene objects={objects} />
    </group>
  );
}
