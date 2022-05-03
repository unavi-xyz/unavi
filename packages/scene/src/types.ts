import { Triplet } from "@react-three/cannon";
import { Primitive, PRIMITIVES } from "./primitives";

export type TreeObjectTypes = "Group" | "Primitive";

export type BaseTreeObject<T extends TreeObjectTypes> = {
  type: T;

  id: string;
  name?: string;

  position: Triplet;
  rotation: Triplet;
  scale: Triplet;

  parentId: string | null;
  children: TreeObject[];
};

export interface GroupTreeObject extends BaseTreeObject<"Group"> {}

export interface PrimitiveTreeObject<T extends Primitive = Primitive>
  extends BaseTreeObject<"Primitive"> {
  primitive: T;
  params: Parameters<typeof PRIMITIVES[T]["Component"]>[0]["params"];
}

export type TreeObject = GroupTreeObject | PrimitiveTreeObject;

export type Scene = {
  tree: TreeObject;
};
