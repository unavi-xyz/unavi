import { SphereCollider } from "@wired-labs/engine";

import { updateNode } from "../../../actions/UpdateNodeAction";
import { useSubscribeValue } from "../../../hooks/useSubscribeValue";
import NumberInput from "../../ui/NumberInput";
import MenuRows from "../MenuRows";

interface Props {
  nodeId: string;
  collider: SphereCollider;
}

export default function SphereColliderComponent({ nodeId, collider }: Props) {
  const radius = useSubscribeValue(collider.radius$);

  return (
    <MenuRows titles={["Radius"]}>
      <NumberInput
        name="Radius"
        value={radius ?? 0}
        step={0.1}
        min={0}
        onChange={(e) => {
          const value = e.target.value;
          if (!value) return;

          const num = parseFloat(value);
          const rounded = Math.round(num * 1000) / 1000;

          collider.radius = rounded;

          updateNode(nodeId, { collider: collider.toJSON() });
        }}
      />
    </MenuRows>
  );
}
