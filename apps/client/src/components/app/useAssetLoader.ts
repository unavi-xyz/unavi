import { useState, useContext, useEffect } from "react";
import { IpfsContext, loadFromIpfs } from "ceramic";
import { Scene, JsonScene } from "3d";

export function useAssetLoader(stringScene: JsonScene) {
  const { ipfs } = useContext(IpfsContext);

  const [scene, setScene] = useState<Scene>();

  useEffect(() => {
    if (!ipfs || !stringScene) return;

    setScene({ ...stringScene, assets: {} });

    //load assets
    Object.keys(stringScene.assets).forEach(async (cid) => {
      const file = await loadFromIpfs(ipfs, cid);

      setScene((prev) => {
        const assets = { ...prev.assets };
        assets[cid] = file;
        prev.assets = assets;
        return { ...prev };
      });
    });
  }, [ipfs, stringScene]);

  return scene;
}
