import { useEffect, useRef, useState } from "react";
import {
  MeshStandardMaterial,
  Texture as ThreeTexture,
  TextureLoader,
} from "three";

import { Params, Texture } from "./types";

interface Props {
  params: Partial<Params>;
  textures: { [key: string]: Texture };
}

export function Material({ params, textures }: Props) {
  const ref = useRef<MeshStandardMaterial>();

  const [texture, setTexture] = useState<ThreeTexture>();

  useEffect(() => {
    if (params?.texture) {
      const texture = textures[params.texture];
      if (!texture) return;

      const loader = new TextureLoader();
      loader.loadAsync(texture.value).then((res) => {
        setTexture(res);
      });
    }
  }, [params, textures]);

  useEffect(() => {
    if (ref.current) ref.current.needsUpdate = true;
  }, [texture]);

  return <meshStandardMaterial ref={ref} map={texture} />;
}
