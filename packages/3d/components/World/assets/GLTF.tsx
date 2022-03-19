import { useEffect, useState } from "react";
import { Triplet } from "@react-three/cannon";
import { GLTF, GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";
import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader";

import { Model } from "../types";

const loader = new GLTFLoader();
const dracoLoader = new DRACOLoader();
dracoLoader.setDecoderPath("https://www.gstatic.com/draco/v1/decoders/");
loader.setDRACOLoader(dracoLoader);

export const gltfDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  scale: [1, 1, 1] as Triplet,
  model: null,
};

interface Props {
  params: typeof gltfDefaultParams;
  models: { [key: string]: Model };
}

export function GLTFModel({ params, models }: Props) {
  const [position] = useState(params.position);
  const [rotation] = useState(params.rotation);
  const [scale] = useState(params.scale);
  const [gltf, setGltf] = useState<GLTF>();

  useEffect(() => {
    if (!params?.model || !models) return;
    const model = models[params?.model];
    if (!model) return;

    loader.loadAsync(model.value).then((res) => {
      setGltf(res);
    });
  }, [params, models]);

  if (!gltf) return <group></group>;

  return (
    <group position={position} rotation={rotation} scale={scale}>
      <primitive object={gltf.scene} />;
    </group>
  );
}
