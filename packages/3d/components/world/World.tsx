import { Sky } from "@react-three/drei";
import { useWorld } from "ceramic";

import { Objects } from "../..";
import { Ground } from "./Ground";

interface Props {
  worldId: string;
}

export function World({ worldId }: Props) {
  const { world } = useWorld(worldId);

  if (!world) return null;

  return (
    <group>
      <ambientLight intensity={0.1} />
      <directionalLight intensity={0.5} />
      <Ground />
      <Sky />
      <Objects objects={Object.values(world.objects)} />;
    </group>
  );
}
