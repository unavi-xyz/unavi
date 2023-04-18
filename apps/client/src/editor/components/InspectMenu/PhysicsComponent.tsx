import { Node } from "@gltf-transform/core";
import { Collider, ColliderExtension, ColliderType } from "@unavi/gltf-extensions";

import { useCollider } from "../../hooks/useExtension";
import { useSubscribe } from "../../hooks/useSubscribe";
import { capitalize } from "../../utils/capitalize";
import SelectMenu from "../ui/SelectMenu";
import BoxColliderComponent from "./collider/BoxColliderComponent";
import CylinderColliderComponent from "./collider/CylinderColliderComponent";
import SphereColliderComponent from "./collider/SphereColliderComponent";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  node: Node;
}

export default function PhysicsComponent({ node }: Props) {
  const collider = useCollider(node);
  const type = useSubscribe(collider, "Type");

  if (!type) return null;

  return (
    <ComponentMenu
      title="Physics"
      onRemove={() => {
        collider?.dispose();
      }}
    >
      <MenuRows titles={["Collider"]}>
        <SelectMenu
          value={capitalize(type)}
          options={["Box", "Sphere", "Cylinder", "Trimesh"]}
          onChange={(e) => {
            if (!node) return;

            const extension = node.getExtension<Collider>(ColliderExtension.EXTENSION_NAME);

            if (extension) {
              extension.setSize(null);
              extension.setHeight(null);
              extension.setRadius(null);
              extension.setMesh(null);

              const value = e.target.value.toLowerCase() as ColliderType;
              extension.setType(value);

              switch (value) {
                case "box": {
                  extension.setSize([1, 1, 1]);
                  break;
                }

                case "sphere": {
                  extension.setRadius(0.5);
                  break;
                }

                case "cylinder": {
                  extension.setHeight(1);
                  extension.setRadius(0.5);
                  break;
                }

                case "trimesh": {
                  const mesh = node.getMesh();
                  extension.setMesh(mesh);
                  break;
                }
              }
            }
          }}
        />
      </MenuRows>

      {type === "box" ? (
        <BoxColliderComponent node={node} />
      ) : type === "sphere" ? (
        <SphereColliderComponent node={node} />
      ) : type === "cylinder" ? (
        <CylinderColliderComponent node={node} />
      ) : null}
    </ComponentMenu>
  );
}
