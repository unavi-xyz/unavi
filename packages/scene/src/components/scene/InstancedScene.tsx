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
      <ambientLight intensity={0.2} />
      <directionalLight intensity={1} position={[-1, 1.5, -2]} />
    </AssetProvider>
  );
}
