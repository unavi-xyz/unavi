import { IModule } from "../../types";
import { COLLIDER_COMPONENTS } from "../components";

interface Props {
  module: IModule;
}

export default function ColliderModule({ module }: Props) {
  const Component = COLLIDER_COMPONENTS[module.variation];
  return <Component {...(module.props as any)} />;
}
