import { useEffect, useState } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls, Sky, TransformControls } from "@react-three/drei";
import { Ground, PARAM_NAMES } from "3d";

import { TOOLS, useStore } from "../state/useStore";

import Objects from "./Objects";

export default function EditorScene() {
  const selected = useStore((state) => state.selected);
  const tool = useStore((state) => state.tool);
  const setUsingGizmo = useStore((state) => state.setUsingGizmo);

  const [enabled, setEnabled] = useState(false);

  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(10, 10, 10);
  }, [camera]);

  function handleMouseDown() {
    setUsingGizmo(true);
  }

  function handleMouseUp() {
    selected.save();
    setUsingGizmo(false);
  }

  useEffect(() => {
    if (!selected) {
      setEnabled(false);
      return;
    }

    const hasType =
      tool === TOOLS.translate
        ? PARAM_NAMES.position in selected.instance.params
        : tool === TOOLS.rotate
        ? PARAM_NAMES.rotation in selected.instance.params
        : tool === TOOLS.scale
        ? PARAM_NAMES.scale in selected.instance.params
        : false;

    setEnabled(hasType);
  }, [selected, tool]);

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Sky />
      <Ground />

      <OrbitControls makeDefault />

      <TransformControls
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        object={selected?.ref}
        enabled={enabled}
        showX={Boolean(selected)}
        showY={Boolean(selected)}
        showZ={Boolean(selected)}
        size={0.7}
        mode={tool}
      />

      <Objects />
    </group>
  );
}
