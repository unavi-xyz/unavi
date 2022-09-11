import { useEditorStore } from "../../store";
import BoxComponent from "./BoxComponent";
import CylinderComponent from "./CylinderComponent";
import MaterialComponent from "./MaterialComponent";
import SphereComponent from "./SphereComponent";

interface Props {
  entityId: string;
}

export default function EntityComponents({ entityId }: Props) {
  const type = useEditorStore((state) => state.scene.entities[entityId].type);

  switch (type) {
    case "Box":
      return (
        <>
          <BoxComponent entityId={entityId} />
          <MaterialComponent entityId={entityId} />
        </>
      );
    case "Sphere":
      return (
        <>
          <SphereComponent entityId={entityId} />
          <MaterialComponent entityId={entityId} />
        </>
      );
    case "Cylinder":
      return (
        <>
          <CylinderComponent entityId={entityId} />
          <MaterialComponent entityId={entityId} />
        </>
      );
    default:
      return null;
  }
}
