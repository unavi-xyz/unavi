import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import BoxMeshComponent from "./BoxMeshComponent";
import CylinderMeshComponent from "./CylinderMeshComponent";
import MaterialComponent from "./MaterialComponent";
import PhysicsComponent from "./PhysicsComponent";
import SphereMeshComponent from "./SphereMeshComponent";

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
          <BoxMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    case "Sphere":
      return (
        <>
          <SphereMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    case "Cylinder":
      return (
        <>
          <CylinderMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    default:
      return (
        <>
          <PhysicsComponent entityId={entityId} />
        </>
      );
  }
}
