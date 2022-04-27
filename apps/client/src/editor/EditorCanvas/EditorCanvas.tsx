import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { OrbitControls } from "@react-three/drei";

import { editorManager } from "../helpers/store";
import { useAutosave } from "../helpers/hooks/useAutosave";
import { useEditorHotkeys } from "../helpers/hooks/useEditorHotkeys";

import EditorScene from "./EditorScene";
import Gizmo from "./Gizmo";

export default function EditorCanvas() {
  useAutosave();
  useEditorHotkeys();

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
