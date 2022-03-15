import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { Player, World } from "3d";

import { useStore } from "../../../helpers/editor/store";

export default function PreviewCanvas() {
  const scene = useStore((state) => state.scene);

  return (
    <Canvas mode="concurrent">
      <Physics>
        <World scene={scene} />
        <Player />
      </Physics>
    </Canvas>
  );
}
