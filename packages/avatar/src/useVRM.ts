import { VRM } from "@pixiv/three-vrm";
import { useGLTF } from "@react-three/drei";
import { useEffect, useState } from "react";

export function useVRM(src: string) {
  const gltf = useGLTF(src);

  const [vrm, setVrm] = useState<VRM>();

  useEffect(() => {
    if (!gltf) return;

    VRM.from(gltf as any).then((res) => {
      //enable shadows
      res.scene.traverse((child) => {
        child.castShadow = true;
        child.receiveShadow = true;
      });

      setVrm(res);
    });
  }, [gltf]);

  useEffect(() => {
    return () => {
      vrm?.dispose();
    };
  }, [vrm]);

  return vrm;
}
