import { useEffect } from "react";
import { Canvas, useThree } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";

import { useStudioStore } from "../../../helpers/studio/store";

import StudioInstancedObject from "./StudioInstancedObject";
import Gizmo from "./Gizmo";

export default function StudioCanvas() {
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <Canvas mode="concurrent" gl={{ preserveDrawingBuffer: true }}>
      <Physics>
        <ambientLight intensity={0.2} />
        <directionalLight intensity={1} position={[-1, 1.5, -2]} />
        <gridHelper args={[100, 100]} />

        <CameraMover />
        <OrbitControls makeDefault />
        <Gizmo />

        <group
          onPointerUp={() => {
            if (useStudioStore.getState().usingGizmo) return;
            useStudioStore.setState({ selectedId: undefined });
          }}
        >
          <Sky />
        </group>

        <StudioInstancedObject object={tree} />
      </Physics>
    </Canvas>
  );
}

function CameraMover() {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(5, 5, 5);
  }, [camera]);

  return null;
}
