import { VRM, VRMLoaderPlugin, VRMUtils } from "@pixiv/three-vrm";
import { useEffect, useState } from "react";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

const loader = new GLTFLoader();
loader.crossOrigin = "anonymous";

loader.register((parser) => {
  return new VRMLoaderPlugin(parser);
});

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
        const newVrm = gltf.userData.vrm as VRM | undefined;

        if (!newVrm) {
          setVrm(undefined);
          throw new Error("VRM not found");
        }

        VRMUtils.removeUnnecessaryJoints(newVrm.scene);
        VRMUtils.removeUnnecessaryVertices(newVrm.scene);

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

  return vrm;
}
