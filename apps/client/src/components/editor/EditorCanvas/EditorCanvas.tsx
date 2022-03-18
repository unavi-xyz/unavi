import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Physics } from "@react-three/cannon";

import { useStore } from "../helpers/store";
import { useAutosave } from "../helpers/useAutosave";
import { useHotkeys } from "../helpers/useHotkeys";

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
