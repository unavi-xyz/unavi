import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Physics } from "@react-three/cannon";
import { useAtom } from "jotai";

import { selectedAtom } from "../../../helpers/editor/state";
import { useAutosave } from "../../../helpers/editor/hooks/useAutosave";

import { EditorWorld } from "./EditorWorld";
import Gizmo from "./Gizmo";

export default function EditorCanvas() {
  const [, setSelected] = useAtom(selectedAtom);

  useAutosave();

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
