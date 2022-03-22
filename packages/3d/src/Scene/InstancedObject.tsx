import { Instance, SceneObjectType } from "./types";
import { SceneObjects } from "./constants";

interface Props {
  instance: Instance<SceneObjectType>;
}

export function InstancedObject({ instance }: Props) {
  const component = { ...SceneObjects[instance.type].component };
  component.props = { properties: instance.properties };
  return component;
}
