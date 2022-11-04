import { Quad, Triplet } from "@wired-labs/engine";

import { updateNode } from "../../actions/UpdateNodeAction";
import { useNode } from "../../hooks/useNode";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { eulerToQuaternion } from "../../utils/eulerToQuaternion";
import { quaternionToEuler } from "../../utils/quaternionToEuler";
import NumberInput from "../ui/NumberInput";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  nodeId: string;
}

export default function TransformComponent({ nodeId }: Props) {
  const position$ = useNode(nodeId, (node) => node.position$);
  const rotation$ = useNode(nodeId, (node) => node.rotation$);
  const scale$ = useNode(nodeId, (node) => node.scale$);

  const position = useSubscribeValue<Triplet>(position$);
  const rotation = useSubscribeValue<Quad>(rotation$);
  const scale = useSubscribeValue<Triplet>(scale$);

  const euler = rotation ? quaternionToEuler(rotation) : null;

  return (
    <ComponentMenu removeable={false}>
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

                    updateNode(nodeId, { position: newPosition });
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

                    const newEuler: Triplet = [...euler];
                    newEuler[i] = radians;

                    const newRotation = eulerToQuaternion(newEuler);

                    updateNode(nodeId, { rotation: newRotation });
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

                    updateNode(nodeId, { scale: newScale });
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
