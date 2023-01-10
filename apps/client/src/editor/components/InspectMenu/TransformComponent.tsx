import { Vec3 } from "engine";

import { useNode } from "../../hooks/useNode";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { eulerToQuaternion } from "../../utils/eulerToQuaternion";
import { quaternionToEuler } from "../../utils/quaternionToEuler";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

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
        <div className="grid grid-cols-3 gap-3">
          {translation?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const name = `translation-${letter}`;

            const rounded = Math.round(value * 10000) / 10000;

            return (
              <div key={name} className="flex space-x-2">
                <label htmlFor={name}>{letter}</label>
                <NumberInput
                  name={name}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 10000) / 10000;

                    const newTranslation: Vec3 = [...translation];
                    newTranslation[i] = rounded;

                    node?.setTranslation(newTranslation);
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-3">
          {euler?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const name = `rotation-${letter}`;

            const degress = Math.round((value * 180) / Math.PI);
            const rounded = Math.round(degress * 10000) / 10000;

            return (
              <div key={name} className="flex space-x-2">
                <label htmlFor={name}>{letter}</label>
                <NumberInput
                  name={name}
                  value={rounded}
                  step={1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 10000) / 10000;
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

        <div className="grid grid-cols-3 gap-3">
          {scale?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const name = `scale-${letter}`;

            const rounded = Math.round(value * 10000) / 10000;

            return (
              <div key={name} className="flex space-x-2">
                <label htmlFor={name}>{letter}</label>
                <NumberInput
                  name={name}
                  value={rounded}
                  step={0.1}
                  onChange={(e) => {
                    // @ts-ignore
                    const value: string | null = e.target.value || null;
                    if (value === null) return;

                    const num = parseFloat(value);
                    const rounded = Math.round(num * 10000) / 10000;

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
