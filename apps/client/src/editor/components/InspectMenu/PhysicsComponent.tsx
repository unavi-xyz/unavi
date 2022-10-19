import {
  BoxCollider,
  Collider,
  CylinderCollider,
  HullCollider,
  MeshCollider,
  SphereCollider,
} from "@wired-labs/engine";

import { updateEntity } from "../../actions/UpdateEntityAction";
import { useEntity } from "../../hooks/useEntity";
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
  entityId: string;
}

export default function PhysicsComponent({ entityId }: Props) {
  const collider$ = useEntity(entityId, (entity) => entity.collider$);
  const collider = useSubscribeValue(collider$);

  if (!collider) return null;

  return (
    <ComponentMenu
      title="Physics"
      onRemove={() => updateEntity(entityId, { collider: null })}
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
                  updateEntity(entityId, { collider: boxCollider.toJSON() });
                  break;
                }

                case "Sphere": {
                  const sphereCollider = new SphereCollider();
                  updateEntity(entityId, { collider: sphereCollider.toJSON() });
                  break;
                }

                case "Cylinder": {
                  const cylinderCollider = new CylinderCollider();
                  updateEntity(entityId, {
                    collider: cylinderCollider.toJSON(),
                  });
                  break;
                }

                case "Hull": {
                  const { engine } = useEditorStore.getState();
                  if (!engine) throw new Error("Engine not found");

                  const hullCollider = new HullCollider();
                  updateEntity(entityId, {
                    collider: hullCollider.toJSON(),
                  });
                  break;
                }

                case "Mesh": {
                  const { engine } = useEditorStore.getState();
                  if (!engine) throw new Error("Engine not found");

                  const meshCollider = new MeshCollider();
                  updateEntity(entityId, {
                    collider: meshCollider.toJSON(),
                  });
                  break;
                }
              }
            }}
          />
        </MenuRows>

        <ColliderComponent entityId={entityId} collider={collider} />
      </>
    </ComponentMenu>
  );
}

function ColliderComponent({
  entityId,
  collider,
}: {
  entityId: string;
  collider: Collider | null;
}) {
  switch (collider?.type) {
    case "box":
      return <BoxColliderComponent entityId={entityId} collider={collider} />;

    case "sphere":
      return (
        <SphereColliderComponent entityId={entityId} collider={collider} />
      );

    case "cylinder":
      return (
        <CylinderColliderComponent entityId={entityId} collider={collider} />
      );

    default:
      return null;
  }
}
