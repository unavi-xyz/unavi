import { useContext, useEffect } from "react";
import { ClientContext } from "matrix";

import { useScene } from "../state/useScene";
import { useStore } from "../state/useStore";

import EditorObject from "./EditorObject";

const AUTOSAVE_INTERVAL = 5000;

export default function Objects() {
  const { client } = useContext(ClientContext);

  const scene = useScene((state) => state.scene);
  const save = useScene((state) => state.save);
  const toJSON = useScene((state) => state.toJSON);
  const fromJSON = useScene((state) => state.fromJSON);

  const roomId = useStore((state) => state.roomId);

  useEffect(() => {
    const interval = setInterval(() => {
      save();
      const json = toJSON();
      const time = Date.now();
      const obj = JSON.stringify({ json, time });

      localStorage.setItem(roomId, obj);
    }, AUTOSAVE_INTERVAL);

    return () => {
      clearInterval(interval);
    };
  }, [scene, roomId, save, toJSON]);

  useEffect(() => {
    const local = JSON.parse(localStorage.getItem(roomId));

    client
      .getStateEvent(roomId, "wired.scene", "wired")
      .then((remote) => {
        if (!local || remote.time > local.time) fromJSON(remote.json);
        else fromJSON(local.json);
      })
      .catch(() => {
        if (local) fromJSON(local.json);
      });
  }, [client, fromJSON, roomId]);

  return (
    <group>
      {Object.values(scene).map((object) => {
        return <EditorObject key={object.id} object={object} />;
      })}
    </group>
  );
}
