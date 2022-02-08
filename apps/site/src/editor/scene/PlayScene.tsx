import { useRef, useState } from "react";
import { Group } from "three";
import { ASSET_NAMES, Player, Scene } from "3d";

import { EditorScene, useScene } from "../state/useScene";

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
  const world = useRef<Group>();

  const scene = useScene((state) => state.scene);

  const [spawn] = useState(getSpawn(scene));

  const objects = Object.values(scene).map((obj) => obj.instance);

  return (
    <group>
      <Player world={world} spawn={spawn} />
      <group ref={world}>
        <Scene objects={objects} />
      </group>
    </group>
  );
}
