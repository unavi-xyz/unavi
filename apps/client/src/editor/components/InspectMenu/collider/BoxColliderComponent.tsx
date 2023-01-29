import { Vec3 } from "engine";

import { useCollider } from "../../../hooks/useExtension";
import { useExtensionAttribute } from "../../../hooks/useExtensionAttribute";
import { useNode } from "../../../hooks/useNode";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function BoxColliderComponent({ nodeId }: Props) {
  const node = useNode(nodeId);
  const collider = useCollider(node);
  const size = useExtensionAttribute(collider, "size") ?? [0, 0, 0];

  return (
    <MenuRows titles={["Width", "Height", "Depth"]}>
      {size.map((value, i) => {
        const name = ["Width", "Height", "Depth"][i];

        return (
          <NumberInput
            key={name}
            name={name}
            value={value}
            step={0.1}
            onChange={(e) => {
              if (!collider) return;

              const value = e.target.value;
              if (!value) return;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              const newSize: Vec3 = [...size];
              newSize[i] = rounded;

              collider.size = newSize;
            }}
          />
        );
      })}
    </MenuRows>
  );
}
