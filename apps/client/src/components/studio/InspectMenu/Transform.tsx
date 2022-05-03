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

  if (!selected) return null;

  function handlePositionChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const oldPosition = selected.position;
    const position: typeof oldPosition = [...oldPosition];
    position[index] = round(Number(value));

    updateObject(selected?.id, { position });
  }

  function handleRotationChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const oldRotation = selected.rotation;
    const rotation: typeof oldRotation = [...oldRotation];
    rotation[index] = degToRad(round(Number(value)));

    updateObject(selected?.id, { rotation });
  }

  function handleScaleChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const oldScale = selected.scale;
    const scale: typeof oldScale = [...oldScale];
    scale[index] = round(Number(value));

    updateObject(selected?.id, { scale });
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center">
        <div className="w-2/3">Position</div>
        <div className="flex items-center space-x-2">
          {selected.position.map((value, i) => (
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
          {selected.rotation.map((value, i) => (
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
        <div className="w-2/3">Scale</div>
        <div className="flex items-center space-x-2">
          {selected.scale.map((value, i) => (
            <div key={i} className="flex items-center space-x-1">
              <div>{XYZ[i]}</div>
              <NumberInput
                updatedValue={String(round(value))}
                onChange={(e) => handleScaleChange(e.currentTarget.value, i)}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
