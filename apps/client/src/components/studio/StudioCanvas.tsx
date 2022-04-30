import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";

import { InstancedObject } from "scene";

import { useStudioStore } from "../../helpers/studio/store";

export default function StudioCanvas() {
  const tree = useStudioStore((state) => state.tree);

  return (
    <Canvas>
      <ambientLight />
      <OrbitControls />
      <Sky />

      <Physics>
        <InstancedObject object={tree} />
      </Physics>
    </Canvas>
  );
}
