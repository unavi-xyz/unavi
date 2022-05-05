import { Entity, IModule } from "../../../types";
import { COLLIDER_COMPONENTS } from "../../../modules";

interface Props {
  module: IModule;
  entity: Entity;
}

export default function ColliderModule({ module, entity }: Props) {
  const Component = COLLIDER_COMPONENTS[module.variation];
  return <Component {...(module.props as any)} transform={entity.transform} />;
}
