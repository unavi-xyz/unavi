import { Group } from "three";

import { Entity, IModule, Transform } from "../../types";
import { MODULE_COMPONENTS } from "../../modules";

interface Props {
  module: IModule;
  entity: Entity;
  entityRef: React.RefObject<Group>;
}

export default function Module(props: Props) {
  const Component = MODULE_COMPONENTS[props.module.type];
  return <Component {...props} />;
}
