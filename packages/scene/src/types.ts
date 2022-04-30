import { Primitive, PRIMITIVES } from "./primitives";

export type TreeObject<T extends Primitive> = {
  id: string;
  name: string;

  parent: TreeObject<Primitive> | undefined;
  children: TreeObject<Primitive>[];

  primitive: T;
  params: Parameters<typeof PRIMITIVES[T]["Component"]>[0]["params"];
};

export type Scene = {
  tree: TreeObject<Primitive>;
};
