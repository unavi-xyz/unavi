import { useSpace } from "ceramic";
import { InstancedScene } from "3d";

import useAssetLoader from "./hooks/useAssetLoader";
import Multiplayer from "./Multiplayer/Multiplayer";

interface Props {
  spaceId: string;
}

export default function World({ spaceId }: Props) {
  const { space } = useSpace(spaceId);
  const scene = useAssetLoader(space?.scene);

  if (!scene) return null;
  return (
    <group>
      <InstancedScene scene={scene} />
      <Multiplayer />
    </group>
  );
}
