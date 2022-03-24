import { Canvas } from "@react-three/fiber";
import { Debug, Physics } from "@react-three/cannon";
import { InstancedScene, Player } from "3d";

import { useStore } from "../helpers/store";

export default function PreviewCanvas() {
  const scene = useStore((state) => state.scene);

  return (
    <Canvas mode="concurrent">
      <Physics>
        <Debug>
          <InstancedScene scene={scene} />
        </Debug>
        <Player />
      </Physics>
    </Canvas>
  );
}
