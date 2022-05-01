import { PRIMITIVES } from "../primitives";
import { TreeObject } from "../types";

interface Props {
  object: TreeObject;
  children: React.ReactNode;
}

export function ObjectComponent({ object, children }: Props) {
  if (object.type === "Group") {
    return <group>{children}</group>;
  }

  if (object.type === "Primitive") {
    const Component = PRIMITIVES[object.primitive]["Component"];

    return (
      <group>
        <Component params={object.params as any} />
        <group>{children}</group>
      </group>
    );
  }

  return null;
}
