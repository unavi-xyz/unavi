import { BoxCollider, Triplet } from "engine";

import { updateNode } from "../../../actions/UpdateNodeAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../MenuRows";

interface Props {
  nodeId: string;
  collider: BoxCollider;
}

export default function BoxColliderComponent({ nodeId, collider }: Props) {
  const size = useSubscribeValue(collider.size$);

  return (
    <MenuRows titles={["Width", "Height", "Depth"]}>
      {size?.map((value, i) => {
        const name = ["Width", "Height", "Depth"][i];

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

              const newSize: Triplet = [...size];
              newSize[i] = rounded;
              collider.size = newSize;

              updateNode(nodeId, { collider: collider.toJSON() });
            }}
          />
        );
      })}
    </MenuRows>
  );
}
