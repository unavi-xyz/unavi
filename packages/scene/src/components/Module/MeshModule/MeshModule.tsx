import { MESH_COMPONENTS } from "../../../modules";
import { IMeshModule } from "../../../types";

interface Props {
  module: IMeshModule;
}

export default function MeshModule({ module }: Props) {
  const Component = MESH_COMPONENTS[module.variation] as any;
  return <Component {...module.props} materialId={module.materialId} />;
}
