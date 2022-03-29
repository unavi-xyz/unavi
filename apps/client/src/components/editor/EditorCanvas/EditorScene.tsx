import { useEffect } from "react";
import { ThreeEvent, useThree } from "@react-three/fiber";
import { Sky } from "@react-three/drei";
import { Ground, SceneContext } from "3d";

import { editorManager, useStore } from "../helpers/store";
import EditorInstance from "./EditorInstance";

export default function EditorScene() {
  const instances = useStore((state) => state.scene.instances);
  const assets = useStore((state) => state.scene.assets);
  const materials = useStore((state) => state.scene.materials);
  const debugMode = useStore((state) => state.debugMode);

  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(6, 6, 6);
  }, [camera.position]);

  function handleVoidClick(e: ThreeEvent<PointerEvent> & MouseEvent) {
    if (e.button !== 0 || useStore.getState().usingGizmo) return;
    e.stopPropagation();

    editorManager.setSelected(undefined);
  }

  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />

      <group onPointerUp={handleVoidClick}>
        <Sky inclination={1} />
        <Ground />
      </group>

      <SceneContext.Provider value={{ assets, materials, debug: debugMode }}>
        {instances &&
          Object.keys(instances).map((id) => {
            return <EditorInstance key={id} id={id} />;
          })}
      </SceneContext.Provider>
    </group>
  );
}
