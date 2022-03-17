import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Physics } from "@react-three/cannon";

import { useStore } from "../../../helpers/editor/store";
import { useAutosave } from "../../../helpers/editor/hooks/useAutosave";
import { useHotkeys } from "../../../helpers/editor/hooks/useHotkeys";

import { EditorWorld } from "./EditorWorld";
import Gizmo from "./Gizmo";

export default function EditorCanvas() {
  useAutosave();
  useHotkeys();

  function handlePointerMiss() {
    useStore.getState().setSelected(null);
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

        <EditorWorld />
      </Physics>
    </Canvas>
  );
}
