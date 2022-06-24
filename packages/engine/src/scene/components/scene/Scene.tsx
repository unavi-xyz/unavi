import { Sky } from "@react-three/drei";

import { IScene } from "../../types";
import { AssetProvider } from "./AssetProvider";
import { Entity } from "./Entity";

interface Props {
  scene: IScene;
}

export function Scene({ scene }: Props) {
  return (
    <AssetProvider assets={scene.assets}>
      <Entity entity={scene.tree} />
      <Sky />
    </AssetProvider>
  );
}
