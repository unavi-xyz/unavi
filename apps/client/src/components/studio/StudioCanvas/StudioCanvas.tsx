import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect } from "react";

import { AssetProvider, MaterialProvider } from "@wired-xr/scene";

import { useStudioStore } from "../../../helpers/studio/store";
import Gizmo from "./Gizmo";
import StudioInstancedEntity from "./StudioInstancedEntity";
import ToggleDebug from "./ToggleDebug";

export default function StudioCanvas() {
  const debug = useStudioStore((state) => state.debug);
  const assets = useStudioStore((state) => state.scene.assets);
  const materials = useStudioStore((state) => state.scene.materials);
  const tree = useStudioStore((state) => state.scene.tree);

  return (
    <Canvas gl={{ preserveDrawingBuffer: true }}>
      <Physics>
        <ToggleDebug enabled={debug}>
          <group>
            <ambientLight intensity={0.2} />
            <directionalLight intensity={1} position={[-1, 1.5, -2]} />
            <gridHelper args={[20, 20]} />

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

            <AssetProvider assets={assets}>
              <MaterialProvider materials={materials}>
                <StudioInstancedEntity entity={tree} />
              </MaterialProvider>
            </AssetProvider>
          </group>
        </ToggleDebug>
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
