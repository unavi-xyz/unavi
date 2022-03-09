import { useEffect } from "react";
import { ThreeEvent, useThree } from "@react-three/fiber";
import { Sky } from "@react-three/drei";
import { useAtom } from "jotai";
import { Ground } from "3d";

import {
  sceneAtom,
  selectedAtom,
  usingGizmoAtom,
} from "../../../helpers/editor/state";

import EditorInstance from "./EditorInstance";

export function EditorWorld() {
  const [scene] = useAtom(sceneAtom);
  const [usingGizmo] = useAtom(usingGizmoAtom);
  const [, setSelected] = useAtom(selectedAtom);

  function handleVoidClick(e: ThreeEvent<MouseEvent>) {
    if (usingGizmo) return;
    e.stopPropagation();
    setSelected(null);
  }

  const { camera } = useThree();

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

      {scene &&
        Object.values(scene).map((instance) => {
          return <EditorInstance key={instance.id} instance={instance} />;
        })}
    </group>
  );
}
