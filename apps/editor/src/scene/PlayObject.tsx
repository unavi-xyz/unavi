import { ASSET_NAMES, Object } from "3d";
import { useScene } from "../state/useScene";

interface Props {
  id: string;
}

export default function PlayObject({ id }: Props) {
  const params = useScene((state) => state.scene[id].params);

  return (
    <group>
      {params.type !== ASSET_NAMES.Spawn && <Object params={params} />}
    </group>
  );
}
