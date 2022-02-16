import { useEffect } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls, Sky } from "@react-three/drei";
import { Ground } from "3d";

import { useStore } from "../hooks/useStore";
import EditorObjects from "./EditorObjects";
import Gizmo from "./Gizmo";

export default function EditorScene() {
  const setSelected = useStore((state) => state.setSelected);

  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(10, 10, 10);
  }, [camera]);

  function handleClick(e) {
    e.stopPropagation();
    setSelected(null);
  }

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Sky />
      <group onClick={handleClick}>
        <Ground />
      </group>

      <OrbitControls makeDefault />
      <Gizmo />

      <EditorObjects />
    </group>
  );
}
