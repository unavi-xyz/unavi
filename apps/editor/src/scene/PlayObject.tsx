import { useEffect, useRef } from "react";
import { ASSET_NAMES, getComponent, SceneObject } from "3d";

interface Props {
  object: SceneObject;
}

export default function PlayObject({ object }: Props) {
  const ref = useRef();

  useEffect(() => {
    object.ref = ref;
    object.load();
  }, [object, ref]);

  return (
    <group ref={ref}>
      {object.type !== ASSET_NAMES.Spawn && getComponent(object)}
    </group>
  );
}
