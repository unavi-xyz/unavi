import { useContext, useEffect, useState } from "react";

import { GLTF, GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";
import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader";

import { SceneContext } from "../SceneContext";
import { fileToDataUrl } from "../helpers";
import { CoreProperties, Properties } from "../types";

const loader = new GLTFLoader();
const dracoLoader = new DRACOLoader();
dracoLoader.setDecoderPath("https://www.gstatic.com/draco/v1/decoders/");
loader.setDRACOLoader(dracoLoader);

export type IGLTF = CoreProperties & Pick<Properties, "scale" | "src">;

export const gltfDefaultProperties: IGLTF = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
  src: undefined,
};

export function GLTFModel({ properties = gltfDefaultProperties }) {
  const [position] = useState(properties.position);
  const [rotation] = useState(properties.rotation);
  const [scale] = useState(properties.scale);

  const { assets } = useContext(SceneContext);
  const [gltf, setGltf] = useState<GLTF>();

  useEffect(() => {
    if (gltf) return;

    if (!properties?.src) {
      setGltf(undefined);
      return;
    }

    const file = assets[properties.src];

    if (!file) {
      setGltf(undefined);
      return;
    }

    fileToDataUrl(file).then((res) => {
      loader.loadAsync(res).then(setGltf);
    });
  }, [assets, properties]);

  if (!gltf) return null;

  return (
    <group position={position} rotation={rotation} scale={scale}>
      <primitive object={gltf.scene} />;
    </group>
  );
}
