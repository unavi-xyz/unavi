import { Vec3 } from "engine";

import { useNode } from "../../hooks/useNode";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { eulerToQuaternion } from "../../utils/eulerToQuaternion";
import { quaternionToEuler } from "../../utils/quaternionToEuler";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function TransformComponent({ nodeId }: Props) {
  const translation = useNodeAttribute(nodeId, "translation");
  const rotation = useNodeAttribute(nodeId, "rotation");
  const scale = useNodeAttribute(nodeId, "scale");
  const node = useNode(nodeId);

  const euler = rotation ? quaternionToEuler(rotation) : null;

  return (
    <ComponentMenu removeable={false}>
      <MenuRows titles={["Translation", "Rotation", "Scale"]}>
        <div className="grid grid-cols-3 gap-x-2">
          {translation?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `translation-${letter}`;

            const rounded = Math.round(value * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <NumberInput
                  id={id}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;

                    const newTranslation: Vec3 = [...translation];
                    newTranslation[i] = rounded;

                    node?.setTranslation(newTranslation);
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-x-2">
          {euler?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `rotation-${letter}`;

            const degress = Math.round((value * 180) / Math.PI);
            const rounded = Math.round(degress * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <NumberInput
                  id={id}
                  value={rounded}
                  step={1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;
                    const radians = (rounded * Math.PI) / 180;

                    const newEuler: Vec3 = [...euler];
                    newEuler[i] = radians;

                    const newRotation = eulerToQuaternion(newEuler);

                    node?.setRotation(newRotation);
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-x-2">
          {scale?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const id = `scale-${letter}`;

            const rounded = Math.round(value * 1000) / 1000;

            return (
              <div key={id} className="flex items-center space-x-1">
                <label htmlFor={id}>{letter}</label>
                <NumberInput
                  id={id}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 1000) / 1000;

                    const newScale: Vec3 = [...scale];
                    newScale[i] = rounded;

                    node?.setScale(newScale);
                  }}
                />
              </div>
            );
          })}
        </div>
      </MenuRows>
    </ComponentMenu>
  );
}
