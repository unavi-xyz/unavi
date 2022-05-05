import { Entity, IModule } from "../../../types";
import { MESH_COMPONENTS } from "../../../modules";

interface Props {
  module: IModule;
  entity: Entity;
}

export default function MeshModule({ module, entity }: Props) {
  const Component = MESH_COMPONENTS[module.variation];
  return <Component {...(module.props as any)} />;
}
