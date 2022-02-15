import { useEffect } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls, Sky } from "@react-three/drei";
import { Ground } from "3d";

import EditorObjects from "./EditorObjects";
import Gizmo from "./Gizmo";

export default function EditorScene() {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(10, 10, 10);
  }, [camera]);

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
      <Sky />

      <OrbitControls makeDefault />
      <Gizmo />

      <EditorObjects />
    </group>
  );
}
