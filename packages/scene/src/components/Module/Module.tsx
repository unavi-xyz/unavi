import { Group } from "three";

import { MODULE_COMPONENTS } from "../../modules";
import { Entity, IModule } from "../../types";

interface Props {
  module: IModule;
  entity: Entity;
  entityRef: React.RefObject<Group>;
}

export default function Module(props: Props) {
  const Component = MODULE_COMPONENTS[props.module.type];
  return <Component {...props} />;
}
