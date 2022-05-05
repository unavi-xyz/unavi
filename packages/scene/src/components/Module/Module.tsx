import { Entity, IModule } from "../../types";
import { MODULE_COMPONENTS } from "../../modules";

interface Props {
  module: IModule;
  entity: Entity;
}

export default function Module({ module, entity }: Props) {
  const Component = MODULE_COMPONENTS[module.type];
  return <Component module={module} entity={entity} />;
}
