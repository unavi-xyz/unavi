import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect } from "react";

import { AssetProvider } from "@wired-xr/scene";

import { useStudioStore } from "../../../helpers/studio/store";
import Gizmo from "./Gizmo";
import StudioInstancedEntity from "./StudioInstancedEntity";
import ToggleDebug from "./ToggleDebug";

export default function StudioCanvas() {
  const debug = useStudioStore((state) => state.debug);
  const assets = useStudioStore((state) => state.scene.assets);
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <Canvas gl={{ preserveDrawingBuffer: true }}>
      <group>
        <ambientLight intensity={0.2} />
        <directionalLight intensity={1} position={[-1, 1.5, -2]} />
        <gridHelper args={[400, 400, 0x666666, 0x999999]} />

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

        <Physics>
          <ToggleDebug enabled={debug}>
            <AssetProvider assets={assets}>
              <StudioInstancedEntity entity={tree} />
            </AssetProvider>
          </ToggleDebug>
        </Physics>
      </group>
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
