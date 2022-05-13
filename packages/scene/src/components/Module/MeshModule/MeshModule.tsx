import { MESH_COMPONENTS } from "../../../modules";
import { IModule } from "../../../types";

interface Props {
  module: IModule;
}

export default function MeshModule({ module }: Props) {
  const Component = MESH_COMPONENTS[module.variation];
  return <Component {...(module.props as any)} />;
}
