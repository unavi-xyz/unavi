import { useEffect } from "react";
import { ThreeEvent, useThree } from "@react-three/fiber";
import { Sky } from "@react-three/drei";
import { Ground, SceneContext } from "3d";

import { editorManager, useStore } from "../helpers/store";
import EditorInstance from "./EditorInstance";

export default function EditorScene() {
  const scene = useStore((state) => state.scene);
  const assets = useStore((state) => state.scene.assets);

  const { camera } = useThree();

  function handleVoidClick(e: ThreeEvent<MouseEvent>) {
    if (useStore.getState().usingGizmo) return;
    e.stopPropagation();

    editorManager.setSelected(undefined);
  }

  useEffect(() => {
    camera.position.set(6, 6, 6);
  }, [camera.position]);

  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />

      <group onPointerUp={handleVoidClick}>
        <Sky inclination={1} />
        <Ground />
      </group>

      <SceneContext.Provider value={{ assets }}>
        {scene &&
          Object.keys(scene.instances).map((id) => {
            return <EditorInstance key={id} id={id} />;
          })}
      </SceneContext.Provider>
    </group>
  );
}
