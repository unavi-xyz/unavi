import { Node } from "@gltf-transform/core";

import { useCollider } from "../../../hooks/useExtension";
import { useSubscribe } from "../../../hooks/useSubscribe";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  node: Node;
}

export default function SphereColliderComponent({ node }: Props) {
  const collider = useCollider(node);
  const radius = useSubscribe(collider, "Radius");

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

          collider.setRadius(rounded);
        }}
      />
    </MenuRows>
  );
}
