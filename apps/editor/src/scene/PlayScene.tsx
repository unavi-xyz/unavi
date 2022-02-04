import { useRef, useState } from "react";
import { Group, Vector3 } from "three";
import { ASSET_NAMES, Player, Scene } from "3d";

import { IScene, useScene } from "../state/useScene";

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

  const params = Object.values(scene).map((obj) => obj.params);

  return (
    <group>
      <Player world={world} spawn={spawn} />
      <Scene scene={params} />
    </group>
  );
}
