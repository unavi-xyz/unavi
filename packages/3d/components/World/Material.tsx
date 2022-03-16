import { useEffect, useRef, useState } from "react";
import { MeshStandardMaterial, Texture, TextureLoader } from "three";

import { Params } from "./types";

interface Props {
  params: Partial<Params>;
}

export function Material({ params }: Props) {
  const ref = useRef<MeshStandardMaterial>();

  const [texture, setTexture] = useState<Texture>();

  useEffect(() => {
    if (params?.texture) {
      const loader = new TextureLoader();
      loader.loadAsync(params?.texture).then((res) => {
        setTexture(res);
      });
    }
  }, [params]);

  useEffect(() => {
    if (ref.current) ref.current.needsUpdate = true;
  }, [texture]);

  return <meshStandardMaterial ref={ref} map={texture} />;
}
