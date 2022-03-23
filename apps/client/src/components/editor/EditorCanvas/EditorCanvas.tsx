import { Canvas } from "@react-three/fiber";
import { Debug, Physics } from "@react-three/cannon";
import { OrbitControls } from "@react-three/drei";

import { editorManager, useStore } from "../helpers/store";
import { useAutosave } from "../helpers/useAutosave";
import { useHotkeys } from "../helpers/useHotkeys";

import EditorScene from "./EditorScene";
import Gizmo from "./Gizmo";

export default function EditorCanvas() {
  useAutosave();
  useHotkeys();

  function handlePointerMiss() {
    editorManager.setSelected(undefined);
  }

  return (
    <Canvas
      mode="concurrent"
      gl={{ preserveDrawingBuffer: true }}
      onPointerMissed={handlePointerMiss}
    >
      <Physics>
        <OrbitControls makeDefault />
        <Gizmo />
        <EditorScene />
      </Physics>
    </Canvas>
  );
}
