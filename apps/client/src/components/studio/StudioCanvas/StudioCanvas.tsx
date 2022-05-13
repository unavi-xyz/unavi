import { Debug, Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect } from "react";

import { useStudioStore } from "../../../helpers/studio/store";
import Gizmo from "./Gizmo";
import StudioInstancedEntity from "./StudioInstancedEntity";

export default function StudioCanvas() {
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <Canvas gl={{ preserveDrawingBuffer: true }}>
      <Physics>
        <Debug>
          <ambientLight intensity={0.2} />
          <directionalLight intensity={1} position={[-1, 1.5, -2]} />
          <gridHelper args={[100, 100]} />

          <CameraMover />
          <OrbitControls makeDefault />
          <Gizmo />

          <group
            onPointerUp={(e: any) => {
              if (e.button !== 0) return;
              if (useStudioStore.getState().usingGizmo) return;
              useStudioStore.setState({ selectedId: undefined });
            }}
          >
            <Sky />
          </group>

          <StudioInstancedEntity entity={tree} />
        </Debug>
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
