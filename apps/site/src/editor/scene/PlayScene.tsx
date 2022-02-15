import { Sky } from "@react-three/drei";
import { Triplet } from "@react-three/cannon";
import { Player, Objects, Ground, ASSET_NAMES, PARAM_NAMES } from "3d";

import { useStore } from "../hooks/useStore";

export default function PlayScene() {
  const objects = useStore((state) => state.objects);

  const instances = Object.values(objects).map((obj) => obj.instance);

  const params = instances.find(
    (obj) => obj.type === ASSET_NAMES.Spawn
  )?.params;

  const spawn: Triplet = params ? params[PARAM_NAMES.position] : [0, 0, 0];

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Sky />
      <Ground />

      <Player spawn={spawn} />
      <Objects objects={instances} />
    </group>
  );
}
