import {
  BoxCollider,
  Collider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "@wired-labs/engine";

import { updateNode } from "../../actions/UpdateNodeAction";
import { useNode } from "../../hooks/useNode";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import { capitalize } from "../../utils/capitalize";
import SelectMenu from "../ui/SelectMenu";
import BoxColliderComponent from "./collider/BoxColliderComponent";
import CylinderColliderComponent from "./collider/CylinderColliderComponent";
import SphereColliderComponent from "./collider/SphereColliderComponent";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  nodeId: string;
}

export default function PhysicsComponent({ nodeId }: Props) {
  const collider$ = useNode(nodeId, (node) => node.collider$);
  const collider = useSubscribeValue(collider$);

  if (!collider) return null;

  return (
    <ComponentMenu
      title="Physics"
      onRemove={() => updateNode(nodeId, { collider: null })}
    >
      <>
        <MenuRows titles={["Collider"]}>
          <SelectMenu
            value={capitalize(collider.type)}
            options={["Box", "Sphere", "Cylinder", "Hull", "Mesh"]}
            onChange={(e) => {
              const value = e.target.value === "None" ? null : e.target.value;

              switch (value) {
                case "Box": {
                  const boxCollider = new BoxCollider();
                  updateNode(nodeId, { collider: boxCollider.toJSON() });
                  break;
                }

                case "Sphere": {
                  const sphereCollider = new SphereCollider();
                  updateNode(nodeId, { collider: sphereCollider.toJSON() });
                  break;
                }

                case "Cylinder": {
                  const cylinderCollider = new CylinderCollider();
                  updateNode(nodeId, {
                    collider: cylinderCollider.toJSON(),
                  });
                  break;
                }

                case "Hull": {
                  const { engine } = useEditorStore.getState();
                  if (!engine) throw new Error("Engine not found");

                  const hullCollider = new HullCollider();
                  updateNode(nodeId, {
                    collider: hullCollider.toJSON(),
                  });
                  break;
                }

                case "Mesh": {
                  const { engine } = useEditorStore.getState();
                  if (!engine) throw new Error("Engine not found");

                  const meshCollider = new MeshCollider();
                  updateNode(nodeId, {
                    collider: meshCollider.toJSON(),
                  });
                  break;
                }
              }
            }}
          />
        </MenuRows>

        <ColliderComponent nodeId={nodeId} collider={collider} />
      </>
    </ComponentMenu>
  );
}

function ColliderComponent({
  nodeId,
  collider,
}: {
  nodeId: string;
  collider: Collider | null;
}) {
  switch (collider?.type) {
    case "box":
      return <BoxColliderComponent nodeId={nodeId} collider={collider} />;

    case "sphere":
      return <SphereColliderComponent nodeId={nodeId} collider={collider} />;

    case "cylinder":
      return <CylinderColliderComponent nodeId={nodeId} collider={collider} />;

    default:
      return null;
  }
}
