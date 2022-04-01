import { useSpace } from "ceramic";
import { InstancedScene } from "3d";

import useAssetLoader from "./hooks/useAssetLoader";

interface Props {
  spaceId: string;
}

export default function World({ spaceId }: Props) {
  const { space } = useSpace(spaceId);
  const scene = useAssetLoader(space?.scene);

  if (!scene) return null;

  return <InstancedScene scene={scene} />;
}
