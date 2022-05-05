import { useEffect, useState } from "react";

import { Entity, IModule } from "../../../types";
import { COLLIDER_COMPONENTS } from "../../../modules";

interface Props {
  module: IModule;
  entity: Entity;
}

export default function ColliderModule({ module, entity }: Props) {
  const [key, setKey] = useState(0);

  //cannon physics objects cant change args once created
  //remount the component every time they change
  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [entity]);

  const Component = COLLIDER_COMPONENTS[module.variation];
  return (
    <Component
      key={key}
      {...(module.props as any)}
      transform={entity.transform}
    />
  );
}
