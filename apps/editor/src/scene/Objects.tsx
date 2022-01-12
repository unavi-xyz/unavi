import { SceneObject } from "3d";
import { useStore } from "../state";

import EditorObject from "./EditorObject";

export default function Objects() {
  const scene = useStore((state) => state.scene);

  return (
    <group>
      {Object.values(scene).map((object: SceneObject) => {
        return <EditorObject key={object.id} object={object} />;
      })}
    </group>
  );
}
