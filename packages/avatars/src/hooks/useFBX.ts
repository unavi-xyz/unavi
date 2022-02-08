import { useEffect, useRef, useState } from "react";
import { Group } from "three";
import { FBXLoader } from "three/examples/jsm/loaders/FBXLoader";

export default function useFBX(url: string) {
  const loader = useRef(new FBXLoader());

  const [fbx, setFbx] = useState<Group>();

  useEffect(() => {
    if (!url) return;
    loader.current.loadAsync(url).then((result) => {
      setFbx(result);
    });
  }, [url]);

  return fbx;
}
