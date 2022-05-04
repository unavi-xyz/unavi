import { IModule } from "../types";
import { MODULE_COMPONENTS } from "./components";

interface Props {
  module: IModule;
}

export default function Module({ module }: Props) {
  const Component = MODULE_COMPONENTS[module.type];
  return <Component module={module} />;
}
