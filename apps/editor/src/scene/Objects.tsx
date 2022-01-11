import { SceneObject } from "3d";
import { useStore } from "../state";

export default function Objects() {
  const scene = useStore((state) => state.scene);

  return (
    <group>
      {Object.values(scene).map((object: SceneObject) => {
        return <group key={object.id}>{object.component}</group>;
      })}
    </group>
  );
}
