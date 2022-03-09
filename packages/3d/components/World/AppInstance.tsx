import { Euler, Vector3 } from "three";

import { Instance } from "./types";
import { InstancedAsset } from "./InstancedAsset";

interface Props {
  instance: Instance;
}

export default function AppInstance({ instance }: Props) {
  const params = instance.params;

  return (
    <group
      position={params?.position && new Vector3().fromArray(params.position)}
      rotation={params?.rotation && new Euler().fromArray(params.rotation)}
      scale={params?.scale && new Vector3().fromArray(params.scale)}
    >
      <InstancedAsset instance={instance} />
    </group>
  );
}
