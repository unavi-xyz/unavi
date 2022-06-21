import { useEffect, useState } from "react";
import { Group } from "three";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader";

const loader = new FBXLoader();

export function useFBX(src: string) {
  const [fbx, setFbx] = useState<Group>();

  useEffect(() => {
    if (!src) {
      setFbx(undefined);
      return;
    }

    async function loadModel() {
      try {
        const fbx = await loader.loadAsync(src);
        setFbx(fbx);
      } catch (e) {
        console.error(e);
        setFbx(undefined);
      }
    }

    loadModel();
  }, [src]);

  return fbx;
}
