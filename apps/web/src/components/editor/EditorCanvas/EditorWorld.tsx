import { useEffect } from "react";
import { ThreeEvent, useThree } from "@react-three/fiber";
import { Sky } from "@react-three/drei";
import { useAtom } from "jotai";
import { Ground } from "3d";

import { usingGizmoAtom } from "../helpers/state";
import { useStore } from "../helpers/store";

import EditorInstance from "./EditorInstance";

export function EditorWorld() {
  const scene = useStore((state) => state.scene);
  const [usingGizmo] = useAtom(usingGizmoAtom);

  const { camera } = useThree();

  function handleVoidClick(e: ThreeEvent<MouseEvent>) {
    if (usingGizmo) return;
    e.stopPropagation();
    useStore.getState().setSelected(null);
  }

  useEffect(() => {
    camera.position.set(6, 6, 6);
  }, [camera.position]);

  return (
    <group>
      <directionalLight intensity={0.7} position={[1, 2, 5]} />
      <ambientLight intensity={0.1} />

      <group onClick={handleVoidClick}>
        <Sky inclination={1} />
        <Ground />
      </group>

      {scene?.instances &&
        Object.keys(scene.instances).map((id) => {
          return <EditorInstance key={id} id={id} />;
        })}
    </group>
  );
}
