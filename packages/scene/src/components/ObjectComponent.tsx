import { Primitive, PRIMITIVES } from "../primitives";
import { TreeObject } from "../types";

interface Props {
  object: TreeObject<Primitive>;
  children: React.ReactNode;
}

export function ObjectComponent({ object, children }: Props) {
  const Component = PRIMITIVES[object.primitive]["Component"];

  if (!Component) return <group>{children}</group>;

  return (
    <group>
      <Component params={object.params as any} />
      <group>{children}</group>
    </group>
  );
}
