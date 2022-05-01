import { TreeObject } from "../types";
import { ObjectComponent } from "./ObjectComponent";

interface Props {
  object: TreeObject;
}

export function InstancedObject({ object }: Props) {
  return (
    <ObjectComponent object={object}>
      {object.children.map((child) => (
        <InstancedObject key={child.id} object={child} />
      ))}
    </ObjectComponent>
  );
}
