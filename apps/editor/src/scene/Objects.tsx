import { useEffect } from "react";

import { useScene } from "../state/useScene";
import { useStore } from "../state/useStore";

import EditorObject from "./EditorObject";

const AUTOSAVE_INTERVAL = 5000;

export default function Objects() {
  const scene = useScene((state) => state.scene);
  const save = useScene((state) => state.save);
  const toJSON = useScene((state) => state.toJSON);
  const fromJSON = useScene((state) => state.fromJSON);

  const id = useStore((state) => state.id);

  useEffect(() => {
    const interval = setInterval(() => {
      save();
      const json = toJSON();
      localStorage.setItem(`${id}-scene`, json);
    }, AUTOSAVE_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [scene, id, save, toJSON]);

  useEffect(() => {
    const scene = localStorage.getItem(`${id}-scene`);
    if (scene) fromJSON(scene);
  }, [fromJSON, id]);

  return (
    <group>
      {Object.values(scene).map((object) => {
        return <EditorObject key={object.id} object={object} />;
      })}
    </group>
  );
}
