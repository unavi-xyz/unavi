import { useEffect, useState } from "react";
import { MeshStandardMaterial } from "three";

import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader";
import { GLTF, GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";
import { fileToDataUrl } from "../../../../../helpers/files";

import { useStore } from "../../../helpers/store";

const loader = new GLTFLoader();
const dracoLoader = new DRACOLoader();
dracoLoader.setDecoderPath("https://www.gstatic.com/draco/v1/decoders/");
loader.setDRACOLoader(dracoLoader);

export default function GLTFInspect() {
  const selected = useStore((state) => state.selected);
  const type = useStore((state) => state.scene.instances[selected?.id]?.type);
  const properties = useStore(
    (state) => state.scene.instances[selected?.id]?.properties
  );

  const [gltf, setGltf] = useState<GLTF>();
  const [materials, setMaterials] = useState<MeshStandardMaterial[]>();

  useEffect(() => {
    if (!properties || type !== "GLTF" || !("src" in properties)) {
      setGltf(undefined);
      return;
    }

    const file = useStore.getState().scene.assets[properties.src];

    fileToDataUrl(file).then((res) => {
      loader.loadAsync(res).then((gltf) => {
        if (!gltf) {
          setMaterials(undefined);
          return;
        }

        const newMaterials = [];
        gltf.scene.traverse((object) => {
          if ("material" in object) {
            //@ts-ignore
            newMaterials.push(object.material);
          }
        });

        setMaterials(newMaterials);
      });
    });
  }, [properties, type]);

  if (!properties || type !== "GLTF" || !materials) return null;

  return (
    <div className="space-y-1">
      <div className="text-xl text-neutral-500 mb-2">Materials</div>

      {materials?.map((material) => {
        return <div key={material.id}>{material.name}</div>;
      })}
    </div>
  );
}
