import { useEffect, useState } from "react";
import { Group } from "three";

import useVRM from "./hooks/useVRM";
import useFBX from "./hooks/useFBX";
import useMixamoAnimation from "./hooks/useMixamoAnimation";

export function Avatar({ firstPerson = false }) {
  // const dance = useFBX(new URL("models/dance.fbx", import.meta.url).href);
  // const jump = useFBX(new URL("models/jump.fbx", import.meta.url).href);
  // const run = useFBX(new URL("models/run.fbx", import.meta.url).href);
  const walk = useFBX(new URL("models/walk.fbx", import.meta.url).href);
  const vrm = useVRM(new URL("models/Tira.vrm", import.meta.url).href);

  const [animation, setAnimation] = useState<Group>();

  useMixamoAnimation(animation, vrm);

  useEffect(() => {
    if (!firstPerson) return;

    vrm?.firstPerson?.setup();
    // camera.layers.enable(vrm.firstPerson.firstPersonOnlyLayer);
  }, [vrm]);

  useEffect(() => {
    if (!walk) return;
    setAnimation(walk);
  }, [walk]);

  return <group>{vrm && <primitive object={vrm.scene} />}</group>;
}
