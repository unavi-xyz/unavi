import { useEffect } from "react";

import { useStore } from "../hooks/useStore";

import EditorObject from "./EditorObject";

const AUTOSAVE_INTERVAL = 2000;

export default function EditorObjects() {
  const scene = useStore((state) => state.scene);
  const sceneId = useStore((state) => state.sceneId);
  const save = useStore((state) => state.save);
  const toJSON = useStore((state) => state.toJSON);
  const fromJSON = useStore((state) => state.fromJSON);

  useEffect(() => {
    const interval = setInterval(() => {
      save();
      const json = toJSON();
      localStorage.setItem(`${sceneId}-scene`, json);
    }, AUTOSAVE_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [scene, sceneId, save, toJSON]);

  useEffect(() => {
    const scene = localStorage.getItem(`${sceneId}-scene`);
    if (scene) fromJSON(scene);
  }, [fromJSON, sceneId]);

  return (
    <group>
      {Object.keys(scene).map((id) => {
        return <EditorObject key={id} id={id} />;
      })}
    </group>
  );
}
