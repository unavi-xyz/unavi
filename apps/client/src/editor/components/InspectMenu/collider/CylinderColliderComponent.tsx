import { CylinderCollider } from "engine";

import { updateNode } from "../../../actions/UpdateNodeAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../MenuRows";

interface Props {
  nodeId: string;
  collider: CylinderCollider;
}

export default function CylinderColliderComponent({ nodeId, collider }: Props) {
  const radius = useSubscribeValue(collider.radius$);
  const height = useSubscribeValue(collider.height$);

  return (
    <MenuRows titles={["Radius", "Height"]}>
      {[radius, height].map((value, i) => {
        const property = i === 0 ? "radius" : "height";
        const name = ["Radius", "Height"][i];

        return (
          <NumberInput
            key={name}
            name={name}
            value={value ?? 0}
            step={0.1}
            onChange={(e) => {
              const value = e.target.value;
              if (!value) return;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              collider[property] = rounded;

              updateNode(nodeId, { collider: collider.toJSON() });
            }}
          />
        );
      })}
    </MenuRows>
  );
}
