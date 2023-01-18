import { Collider, ColliderExtension, ColliderType } from "engine";

import { useCollider } from "../../hooks/useExtension";
import { useExtensionAttribute } from "../../hooks/useExtensionAttribute";
import { useNode } from "../../hooks/useNode";
import { capitalize } from "../../utils/capitalize";
import SelectMenu from "../ui/SelectMenu";
import BoxColliderComponent from "./collider/BoxColliderComponent";
import CylinderColliderComponent from "./collider/CylinderColliderComponent";
import SphereColliderComponent from "./collider/SphereColliderComponent";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function PhysicsComponent({ nodeId }: Props) {
  const node = useNode(nodeId);
  const collider = useCollider(node);
  const type = useExtensionAttribute(collider, "type");

  if (!type) return null;

  return (
    <ComponentMenu
      title="Physics"
      onRemove={() => {
        node?.setExtension(ColliderExtension.EXTENSION_NAME, null);
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
              extension.size = null;
              extension.height = null;
              extension.radius = null;
              extension.mesh = null;

              const value = e.target.value.toLowerCase() as ColliderType;
              extension.type = value;

              switch (value) {
                case "box": {
                  extension.size = [1, 1, 1];
                  break;
                }

                case "sphere": {
                  extension.radius = 0.5;
                  break;
                }

                case "cylinder": {
                  extension.height = 1;
                  extension.radius = 0.5;
                  break;
                }

                case "trimesh": {
                  const mesh = node.getMesh();
                  extension.mesh = mesh;
                  break;
                }
              }
            }
          }}
        />
      </MenuRows>

      {type === "box" ? (
        <BoxColliderComponent nodeId={nodeId} />
      ) : type === "sphere" ? (
        <SphereColliderComponent nodeId={nodeId} />
      ) : type === "cylinder" ? (
        <CylinderColliderComponent nodeId={nodeId} />
      ) : null}
    </ComponentMenu>
  );
}
