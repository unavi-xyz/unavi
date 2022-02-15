import { useEffect } from "react";
import { useStore } from "../hooks/useStore";

import EditorObject from "./EditorObject";

const AUTOSAVE_INTERVAL = 2000;

export default function EditorObjects() {
  const save = useStore((state) => state.save);
  const toJSON = useStore((state) => state.toJSON);
  const fromJSON = useStore((state) => state.fromJSON);
  const objects = useStore((state) => state.objects);
  const sceneId = useStore((state) => state.sceneId);

  useEffect(() => {
    const interval = setInterval(() => {
      const json = toJSON();
      localStorage.setItem(`${sceneId}-scene`, json);
    }, AUTOSAVE_INTERVAL);

    return () => clearInterval(interval);
  }, [save, sceneId, toJSON]);

  useEffect(() => {
    const prev = localStorage.getItem(`${sceneId}-scene`);
    if (prev) fromJSON(prev);
  }, [fromJSON, sceneId]);

  return (
    <group>
      {Object.keys(objects).map((id) => {
        return <EditorObject key={id} id={id} />;
      })}
    </group>
  );
}
