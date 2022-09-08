import { useEditorStore } from "../../store";
import BoxComponent from "./BoxComponent";
import CylinderComponent from "./CylinderComponent";
import SphereComponent from "./SphereComponent";

interface Props {
  entityId: string;
}

export default function EntityComponent({ entityId }: Props) {
  const type = useEditorStore((state) => state.tree[entityId].type);

  switch (type) {
    case "Box":
      return <BoxComponent entityId={entityId} />;
    case "Sphere":
      return <SphereComponent entityId={entityId} />;
    case "Cylinder":
      return <CylinderComponent entityId={entityId} />;
    default:
      return null;
  }
}
