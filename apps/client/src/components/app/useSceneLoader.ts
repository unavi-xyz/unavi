import { useState, useContext, useEffect } from "react";
import { IpfsContext, loadFromIpfs } from "ceramic";
import { Scene } from "3d";

export function useSceneLoader(scene: Scene) {
  const { ipfs } = useContext(IpfsContext);

  const [textures, setTextures] = useState(scene?.textures ?? {});
  const [models, setModels] = useState(scene?.models ?? {});

  useEffect(() => {
    if (!scene || !ipfs) return;

    //load textures
    Object.keys(scene.textures).forEach(async (cid) => {
      const value = await loadFromIpfs(ipfs, cid);
      setTextures((prev) => {
        prev[cid] = { value, name: "" };
        return { ...prev };
      });
    });

    //load models
    Object.keys(scene.models).forEach(async (cid) => {
      const value = await loadFromIpfs(ipfs, cid);
      setModels((prev) => {
        prev[cid] = { value, name: "" };
        return { ...prev };
      });
    });
  }, [ipfs, scene]);

  return { textures, models };
}
