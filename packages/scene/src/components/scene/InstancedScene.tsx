import { Sky } from "@react-three/drei";

import { Scene } from "../../types";
import { AssetProvider } from "./AssetProvider";
import { InstancedEntity } from "./InstancedEntity";

interface Props {
  scene: Scene;
}

export function InstancedScene({ scene }: Props) {
  return (
    <AssetProvider assets={scene.assets}>
      <InstancedEntity entity={scene.tree} />
      <Sky />
    </AssetProvider>
  );
}
