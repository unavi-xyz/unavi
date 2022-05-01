import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";

import { useStudioStore } from "../../../helpers/studio/store";

import StudioInstancedObject from "./StudioInstancedObject";
import Gizmo from "./Gizmo";

export default function StudioCanvas() {
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <Canvas gl={{ preserveDrawingBuffer: true }}>
      <ambientLight intensity={0.2} />
      <directionalLight intensity={1} position={[-1, 1.5, -2]} />

      <OrbitControls makeDefault />
      <Gizmo />

      <group
        onPointerUp={() => {
          if (useStudioStore.getState().usingGizmo) return;
          useStudioStore.setState({ selected: undefined });
        }}
      >
        <Sky />
      </group>

      <Physics>
        <StudioInstancedObject object={tree} />
      </Physics>
    </Canvas>
  );
}
