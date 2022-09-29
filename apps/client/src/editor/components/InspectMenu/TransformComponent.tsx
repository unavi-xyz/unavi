import { Quad, Triplet } from "@wired-labs/engine";

import { updateEntity } from "../../actions/UpdateEntityAction";
import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { eulerToQuaternion } from "../../utils/eulerToQuaternion";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function TransformComponent({ entityId }: Props) {
  const position$ = useEntity(entityId, (entity) => entity.position$);
  const rotation$ = useEntity(entityId, (entity) => entity.rotation$);
  const scale$ = useEntity(entityId, (entity) => entity.scale$);

  const position = useSubscribeValue<Triplet>(position$);
  const rotation = useSubscribeValue<Quad>(rotation$);
  const scale = useSubscribeValue<Triplet>(scale$);

  return (
    <ComponentMenu>
      <MenuRows titles={["Position", "Rotation", "Scale"]}>
        <div className="grid grid-cols-3 gap-3">
          {position?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const name = `position-${letter}`;

            const rounded = Math.round(value * 1000) / 1000;

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
                    const rounded = Math.round(num * 1000) / 1000;

                    const newPosition: Triplet = [...position];
                    newPosition[i] = rounded;

                    updateEntity(entityId, {
                      position: newPosition,
                    });
                  }}
                />
              </div>
            );
          })}
        </div>

        <div className="grid grid-cols-3 gap-3">
          {rotation?.map((value, i) => {
            const letter = ["X", "Y", "Z"][i];
            const name = `rotation-${letter}`;

            const degress = Math.round((value * 180) / Math.PI);
            const rounded = Math.round(degress * 1000) / 1000;

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
                    const rounded = Math.round(num * 1000) / 1000;
                    const radians = (rounded * Math.PI) / 180;

                    const euler = [...rotation];
                    euler[i] = radians;

                    const newRotation = eulerToQuaternion(euler);
                    newRotation[i] = radians;

                    updateEntity(entityId, {
                      rotation: newRotation,
                    });
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

            const rounded = Math.round(value * 1000) / 1000;

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
                    const rounded = Math.round(num * 1000) / 1000;

                    const newScale: Triplet = [...scale];
                    newScale[i] = rounded;

                    updateEntity(entityId, { scale: newScale });
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
