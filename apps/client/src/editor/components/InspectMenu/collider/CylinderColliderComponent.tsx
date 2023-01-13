import { useCollider } from "../../../hooks/useExtension";
import { useExtensionAttribute } from "../../../hooks/useExtensionAttribute";
import { useNode } from "../../../hooks/useNode";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function CylinderColliderComponent({ nodeId }: Props) {
  const node = useNode(nodeId);
  const collider = useCollider(node);
  const radius = useExtensionAttribute(collider, "radius");
  const height = useExtensionAttribute(collider, "height");

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
              if (!collider) return;

              const value = e.target.value;
              if (!value) return;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              collider[property] = rounded;
            }}
          />
        );
      })}
    </MenuRows>
  );
}
