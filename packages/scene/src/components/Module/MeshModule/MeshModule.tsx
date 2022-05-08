import { IModule } from "../../../types";
import { MESH_COMPONENTS } from "../../../modules";

interface Props {
  module: IModule;
}

export default function MeshModule({ module }: Props) {
  const Component = MESH_COMPONENTS[module.variation];
  return <Component {...(module.props as any)} />;
}
