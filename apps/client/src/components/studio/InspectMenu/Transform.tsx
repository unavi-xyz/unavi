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
  const updateEntity = useStudioStore((state) => state.updateEntity);

  if (!selected) return null;

  const { position, rotation, scale } = selected.transform;

  function handlePositionChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const newPosition: typeof position = [...position];
    newPosition[index] = round(Number(value));

    const newTransform = {
      position: newPosition,
      rotation,
      scale,
    };

    updateEntity(selected?.id, { transform: newTransform });
  }

  function handleRotationChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const newRotation: typeof rotation = [...rotation];
    newRotation[index] = degToRad(round(Number(value)));

    const newTransform = {
      position,
      rotation: newRotation,
      scale,
    };

    updateEntity(selected?.id, { transform: newTransform });
  }

  function handleScaleChange(value: string, index: number) {
    if (isNaN(Number(value)) || value === "" || !selected) return;

    const newScale: typeof scale = [...scale];
    newScale[index] = round(Number(value));

    const newTransform = {
      position,
      rotation,
      scale: newScale,
    };

    updateEntity(selected?.id, { transform: newTransform });
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center">
        <div className="w-2/3">Position</div>
        <div className="flex items-center space-x-2">
          {position.map((value, i) => (
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
          {rotation.map((value, i) => (
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
          {scale.map((value, i) => (
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
