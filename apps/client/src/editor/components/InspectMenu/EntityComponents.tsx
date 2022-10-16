import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import MaterialComponent from "./MaterialComponent";
import BoxMeshComponent from "./mesh/BoxMeshComponent";
import CylinderMeshComponent from "./mesh/CylinderMeshComponent";
import GLTFMeshComponent from "./mesh/GLTFMeshComponent";
import SphereMeshComponent from "./mesh/SphereMeshComponent";
import PhysicsComponent from "./PhysicsComponent";

interface Props {
  entityId: string;
}

export default function EntityComponents({ entityId }: Props) {
  const mesh$ = useEntity(entityId, (entity) => entity.mesh$);
  const mesh = useSubscribeValue(mesh$);

  switch (mesh?.type) {
    case "Box": {
      return (
        <>
          <BoxMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }

    case "Sphere": {
      return (
        <>
          <SphereMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }

    case "Cylinder": {
      return (
        <>
          <CylinderMeshComponent entityId={entityId} mesh={mesh} />
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }

    case "glTF": {
      return (
        <>
          <GLTFMeshComponent entityId={entityId} mesh={mesh} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }

    case "Primitive": {
      return (
        <>
          <MaterialComponent entityId={entityId} />
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }

    default: {
      return (
        <>
          <PhysicsComponent entityId={entityId} />
        </>
      );
    }
  }
}
