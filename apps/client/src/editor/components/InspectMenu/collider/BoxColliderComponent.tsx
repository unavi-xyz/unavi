import { Node } from "@gltf-transform/core";
import { Vec3 } from "engine";

import { useCollider } from "../../../hooks/useExtension";
import { useSubscribe } from "../../../hooks/useSubscribe";
import EditorInput from "../../ui/EditorInput";
import MenuRows from "../ui/MenuRows";

interface Props {
  node: Node;
}

export default function BoxColliderComponent({ node }: Props) {
  const collider = useCollider(node);
  const size = useSubscribe(collider, "Size") ?? [0, 0, 0];

  return (
    <MenuRows titles={["Width", "Height", "Depth"]}>
      {size.map((value, i) => {
        const name = ["Width", "Height", "Depth"][i];

        return (
          <EditorInput
            type="number"
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

              collider.setSize(newSize);
            }}
          />
        );
      })}
    </MenuRows>
  );
}
