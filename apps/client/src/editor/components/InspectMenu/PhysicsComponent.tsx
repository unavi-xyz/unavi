import { OMICollider } from "@wired-labs/engine";

import { useEntity } from "../../hooks/useEntity";
import { useEditorStore } from "../../store";
import SelectMenu from "../ui/SelectMenu";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./MenuRows";

interface Props {
  entityId: string;
}

export default function PhysicsComponent({ entityId }: Props) {
  const collider$ = useEntity(entityId, (entity) => entity.collider$);

  const colliderValue = colliderToOption(collider);

  return (
    <ComponentMenu title="Physics">
      <MenuRows titles={["Collider"]}>
        <SelectMenu
          value={colliderValue}
          options={["None", "Box", "Sphere", "Cylinder"]}
          onChange={(e) => {
            const value = colliderOptionToValue(e.target.value);
            const { scene } = useEditorStore.getState();
            const entity = scene.entities[entityId];

            switch (value) {
              case "box":
                entity.collider = {
                  type: "box",
                  extents: [1, 1, 1],
                };
                break;
              case "sphere":
                entity.collider = {
                  type: "sphere",
                  radius: 0.5,
                };
                break;
              case "cylinder":
                entity.collider = {
                  type: "cylinder",
                  radiusTop: 0.5,
                  radiusBottom: 0.5,
                  height: 1,
                };
                break;
              default:
                entity.collider = null;
            }

            setPhysics(entity);
          }}
        />
      </MenuRows>
    </ComponentMenu>
  );
}

function colliderToOption(collider: OMICollider | null) {
  switch (collider?.type) {
    case "box":
      return "Box";
    case "sphere":
      return "Sphere";
    case "cylinder":
      return "Cylinder";
    default:
      return "None";
  }
}

function colliderOptionToValue(option: string) {
  switch (option) {
    case "Box":
      return "box";
    case "Sphere":
      return "sphere";
    case "Cylinder":
      return "cylinder";
    default:
      return null;
  }
}
