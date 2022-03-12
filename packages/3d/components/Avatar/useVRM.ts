import { useEffect, useRef, useState } from "react";
import { VRM } from "@pixiv/three-vrm";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

export function useVRM(src: string) {
  const loader = useRef(new GLTFLoader());
  const [vrm, setVrm] = useState<VRM>();

  useEffect(() => {
    if (!src || vrm) return;
    loader.current.loadAsync(src).then(async (gltf) => {
      const vrm = await VRM.from(gltf);
      setVrm(vrm);
    });
  }, [src]);

  useEffect(() => {
    return () => {
      vrm?.dispose();
    };
  }, [vrm]);

  return vrm;
}
