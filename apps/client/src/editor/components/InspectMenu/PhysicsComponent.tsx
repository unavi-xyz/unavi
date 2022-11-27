import {
  AutoCollider,
  BoxCollider,
  Collider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "engine";

import { updateNode } from "../../actions/UpdateNodeAction";
import { useNode } from "../../hooks/useNode";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { capitalize } from "../../utils/capitalize";
import { updateGltfColliders } from "../../utils/updateGltfColliders";
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
  const node = useNode(nodeId);
  const collider = useSubscribeValue(node?.collider$);
  const meshId = useSubscribeValue(node?.meshId$);

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
            options={["Auto", "Box", "Sphere", "Cylinder", "Mesh"]}
            onChange={(e) => {
              const value = e.target.value === "None" ? null : e.target.value;

              switch (value) {
                case "Auto": {
                  const autoCollider = new AutoCollider();
                  updateNode(nodeId, { collider: autoCollider.toJSON() });
                  break;
                }

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
                  updateNode(nodeId, { collider: cylinderCollider.toJSON() });
                  break;
                }

                case "Hull": {
                  if (!meshId) break;
                  const hullCollider = new HullCollider();
                  hullCollider.meshId = meshId;
                  updateNode(nodeId, { collider: hullCollider.toJSON() });
                  break;
                }

                case "Mesh": {
                  if (!meshId) break;
                  const meshCollider = new MeshCollider();
                  meshCollider.meshId = meshId;
                  updateNode(nodeId, { collider: meshCollider.toJSON() });
                  break;
                }
              }

              updateGltfColliders(nodeId);
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
