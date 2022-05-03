import { useAtomValue } from "jotai";
import { degToRad, radToDeg } from "three/src/math/MathUtils";

import { selectedAtom } from "../../../helpers/studio/atoms";
import { useStudioStore } from "../../../helpers/studio/store";

import NumberInput from "./NumberInput";

function round(value: number, digits = 3) {
  return Math.round(value * 10 ** digits) / 10 ** digits;
}

const XYZ = ["X", "Y", "Z"];

export default function Transform() {
  const selected = useAtomValue(selectedAtom);
  const updateObject = useStudioStore((state) => state.updateObject);

  if (!selected?.params?.position) return null;

  function handlePositionChange(value: string, index: number) {
    if (!selected?.params?.position) return;
    if (isNaN(Number(value)) || value === "") return;

    const oldPosition = selected.params.position;
    const position: typeof oldPosition = [...oldPosition];
    position[index] = round(Number(value));

    updateObject(selected?.id, { position });
  }

  function handleRotationChange(value: string, index: number) {
    if (!selected?.params?.rotation) return;
    if (isNaN(Number(value)) || value === "") return;

    const oldRotation = selected.params.rotation;
    const rotation: typeof oldRotation = [...oldRotation];
    rotation[index] = degToRad(round(Number(value)));

    updateObject(selected?.id, { rotation });
  }

  function handleScaleChange(value: string, index: number) {
    // if (!selected?.params?.scale) return;
    // if (isNaN(Number(value)) || value === "") return;
    // const oldScale = selected.params.scale;
    // const scale: typeof oldScale = [...oldScale];
    // scale[index] = degToRad(round(Number(value)));
    // updateObject(selected?.id, { scale });
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center">
        <div className="w-2/3">Position</div>
        <div className="flex items-center space-x-2">
          {selected.params.position.map((value, i) => (
            <div key={i} className="flex items-center space-x-1">
              <div>{XYZ[i]}</div>
              <NumberInput
                updatedValue={String(round(value))}
                onChange={(e) => handlePositionChange(e.currentTarget.value, i)}
              />
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center">
        <div className="w-2/3">Rotation</div>
        <div className="flex items-center space-x-2">
          {selected.params.rotation.map((value, i) => (
            <div key={i} className="flex items-center space-x-1">
              <div>{XYZ[i]}</div>
              <NumberInput
                updatedValue={String(round(radToDeg(value)))}
                onChange={(e) => handleRotationChange(e.currentTarget.value, i)}
              />
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center">
        {/* <div className="w-2/3">Scale</div>
        <div className="flex items-center space-x-2">
          {selected.params.scale.map((value, i) => (
            <div key={i} className="flex items-center space-x-1">
              <div>{XYZ[i]}</div>
              <NumberInput
                updatedValue={String(round(value))}
                onChange={(e) => handleScaleChange(e.currentTarget.value, i)}
              />
            </div>
          ))}
        </div> */}
      </div>
    </div>
  );
}
