import { useState, useContext, useEffect } from "react";
import { IpfsContext, loadImage } from "ceramic";
import { Scene } from "3d";

export function useSceneLoader(scene: Scene) {
  const { ipfs } = useContext(IpfsContext);

  const [textures, setTextures] = useState(scene?.textures ?? {});

  useEffect(() => {
    if (!scene || !ipfs) return;

    Object.keys(scene.textures).forEach(async (cid) => {
      const value = await loadImage(ipfs, cid);
      setTextures((prev) => {
        prev[cid] = { value, name: "" };
        return { ...prev };
      });
    });
  }, [ipfs, scene]);

  return textures;
}
