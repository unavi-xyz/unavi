import { useEffect, useRef, useState } from "react";
import { VRM } from "@pixiv/three-vrm";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

export default function useVRM(url: string) {
  const loader = useRef(new GLTFLoader());
  const [vrm, setVrm] = useState<VRM>();

  useEffect(() => {
    if (!url) return;
    loader.current.loadAsync(url).then(async (gltf) => {
      const vrm = await VRM.from(gltf);
      setVrm(vrm);
    });
  }, [url]);

  return vrm;
}
