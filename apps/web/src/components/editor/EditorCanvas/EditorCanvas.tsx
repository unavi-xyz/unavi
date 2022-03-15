import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Physics } from "@react-three/cannon";

import { useStore } from "../../../helpers/editor/store";
import { useAutosave } from "../../../helpers/editor/hooks/useAutosave";
import { useHotkeys } from "../../../helpers/editor/hooks/useHotkeys";

import { EditorWorld } from "./EditorWorld";
import Gizmo from "./Gizmo";
import { useEffect } from "react";

export default function EditorCanvas() {
  const setSelected = useStore((state) => state.setSelected);

  useAutosave();
  useHotkeys();

  return (
    <Canvas
      mode="concurrent"
      gl={{ preserveDrawingBuffer: true }}
      onPointerMissed={() => {
        setSelected(null);
      }}
    >
      <Physics>
        <OrbitControls makeDefault />
        <Gizmo />

        <EditorWorld />
      </Physics>
    </Canvas>
  );
}
