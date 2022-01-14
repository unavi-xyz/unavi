import { useEffect } from "react";

import { useScene } from "../hooks/useScene";
import { useStore } from "../hooks/useStore";

import EditorObject from "./EditorObject";

const AUTOSAVE_INTERVAL = 5000;

export default function Objects() {
  const scene = useScene((state) => state.scene);
  const save = useScene((state) => state.save);
  const toJSON = useScene((state) => state.toJSON);
  const fromJSON = useScene((state) => state.fromJSON);

  const roomId = useStore((state) => state.roomId);

  useEffect(() => {
    const interval = setInterval(() => {
      save();
      const json = toJSON();
      localStorage.setItem(roomId, json);
    }, AUTOSAVE_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [scene, roomId, save, toJSON]);

  useEffect(() => {
    const json = localStorage.getItem(roomId);
    fromJSON(json);
  }, [fromJSON, roomId]);

  return (
    <group>
      {Object.values(scene).map((object) => {
        return <EditorObject key={object.id} object={object} />;
      })}
    </group>
  );
}
