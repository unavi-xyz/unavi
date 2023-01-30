import { useCollider } from "../../../hooks/useExtension";
import { useExtensionAttribute } from "../../../hooks/useExtensionAttribute";
import { useNode } from "../../../hooks/useNode";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function SphereColliderComponent({ nodeId }: Props) {
  const node = useNode(nodeId);
  const collider = useCollider(node);
  const radius = useExtensionAttribute(collider, "radius");

  return (
    <MenuRows titles={["Radius"]}>
      <NumberInput
        name="Radius"
        value={radius ?? 0}
        step={0.1}
        min={0}
        onChange={(e) => {
          if (!collider) return;

          const value = e.target.value;
          if (!value) return;

          const num = parseFloat(value);
          const rounded = Math.round(num * 1000) / 1000;

          collider.radius = rounded;
        }}
      />
    </MenuRows>
  );
}
