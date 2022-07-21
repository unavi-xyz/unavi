import { Physics } from "@react-three/cannon";
import { OrbitControls, Sky } from "@react-three/drei";
import { Canvas, useThree } from "@react-three/fiber";
import { useEffect } from "react";

import { AssetProvider } from "@wired-xr/engine";

import { useStudioStore } from "../../../studio/store";
import Gizmo from "./Gizmo";
import StudioInstancedEntity from "./StudioInstancedEntity";
import ToggleDebug from "./ToggleDebug";

export default function StudioCanvas() {
  // const debug = useStudioStore((state) => state.debug);
  // const grid = useStudioStore((state) => state.grid);
  // const assets = useStudioStore((state) => state.scene.assets);
  // const tree = useStudioStore((state) => state.scene.tree);

  return <canvas className="w-full h-full" />;

  // <Canvas gl={{ preserveDrawingBuffer: true }} shadows>
  //   <group>
  //     {grid && <gridHelper args={[1000, 1000, 0x666666, 0x999999]} />}

  //     <CameraMover />
  //     <OrbitControls makeDefault />
  //     <Gizmo />

  //     <group
  //       onPointerUp={(e: any) => {
  //         if (e.button !== 0) return;
  //         if (useStudioStore.getState().usingGizmo) return;
  //         useStudioStore.setState({ selectedId: undefined });
  //       }}
  //     >
  //       <Sky />
  //     </group>

  //     <Physics>
  //       <ToggleDebug enabled={debug}>
  //         <AssetProvider assets={assets}>
  //           <StudioInstancedEntity entity={tree} />
  //         </AssetProvider>
  //       </ToggleDebug>
  //     </Physics>
  //   </group>
  // </Canvas>
}
