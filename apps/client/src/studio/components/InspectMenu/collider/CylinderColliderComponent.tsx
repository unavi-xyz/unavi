import { Node } from "@gltf-transform/core";

import { useCollider } from "../../../hooks/useExtension";
import { useSubscribe } from "../../../hooks/useSubscribe";
import StudioInput from "../../ui/StudioInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  node: Node;
}

export default function CylinderColliderComponent({ node }: Props) {
  const collider = useCollider(node);
  const radius = useSubscribe(collider, "Radius");
  const height = useSubscribe(collider, "Height");

  return (
    <MenuRows titles={["Radius", "Height"]}>
      {[radius, height].map((value, i) => {
        const property = i === 0 ? "radius" : "height";
        const name = ["Radius", "Height"][i];

        return (
          <StudioInput
            key={name}
            name={name}
            type="number"
            value={value ?? 0}
            step={0.1}
            onChange={(e) => {
              if (!collider) return;

              const value = e.target.value;
              if (!value) return;

              const num = parseFloat(value);
              const rounded = Math.round(num * 1000) / 1000;

              if (property === "radius") collider.setRadius(rounded);
              if (property === "height") collider.setHeight(rounded);
            }}
          />
        );
      })}
    </MenuRows>
  );
}
