import { useEffect, useRef, useState } from "react";
import { Group } from "three";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader";

export default function useFBX(url: string) {
  const loader = useRef(new FBXLoader());
  const [fbx, setFBX] = useState<Group>();

  useEffect(() => {
    if (!url || fbx) return;
    loader.current.loadAsync(url).then((loaded) => {
      setFBX(loaded);
    });
  }, [url]);

  return fbx;
}
