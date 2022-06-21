import { VRM } from "@pixiv/three-vrm";
import { useEffect, useState } from "react";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

const loader = new GLTFLoader();

export function useVRM(src: string) {
  const [vrm, setVrm] = useState<VRM>();

  useEffect(() => {
    if (!src) {
      setVrm(undefined);
      return;
    }

    async function loadModel() {
      try {
        const gltf = await loader.loadAsync(src);
        const newVrm = await VRM.from(gltf);

        //enable shadows
        newVrm.scene.traverse((child) => {
          child.castShadow = true;
          child.receiveShadow = true;
        });

        setVrm(newVrm);
      } catch (e) {
        console.error(e);
        setVrm(undefined);
      }
    }

    loadModel();
  }, [src]);

  useEffect(() => {
    return () => {
      vrm?.dispose();
    };
  }, [vrm]);

  return vrm;
}
