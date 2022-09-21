import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import BoxComponent from "./BoxComponent";
import CylinderComponent from "./CylinderComponent";
import MaterialComponent from "./MaterialComponent";
import PhysicsComponent from "./PhysicsComponent";
import SphereComponent from "./SphereComponent";

interface Props {
  entityId: string;
}

export default function EntityComponents({ entityId }: Props) {
  const mesh$ = useEntity(entityId, (entity) => entity.mesh$);
  const mesh = useSubscribeValue(mesh$);

  switch (mesh?.type) {
    case "Box":
      return (
        <>
          <BoxComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          {/* <PhysicsComponent entityId={entityId} /> */}
        </>
      );
    case "Sphere":
      return (
        <>
          <SphereComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          {/* <PhysicsComponent entityId={entityId} /> */}
        </>
      );
    case "Cylinder":
      return (
        <>
          <CylinderComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          {/* <PhysicsComponent entityId={entityId} /> */}
        </>
      );
    default:
      return <>{/* <PhysicsComponent entityId={entityId} /> */}</>;
  }
}
