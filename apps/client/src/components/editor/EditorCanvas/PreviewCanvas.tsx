import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { InstancedScene, Player } from "3d";

import { useStore } from "../helpers/store";

export default function PreviewCanvas() {
  const scene = useStore((state) => state.scene);

  return (
    <Canvas mode="concurrent">
      <Physics>
        <InstancedScene scene={scene} />
        <Player />
      </Physics>
    </Canvas>
  );
}
