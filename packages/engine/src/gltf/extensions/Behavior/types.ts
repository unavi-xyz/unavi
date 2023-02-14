import { Vec2, Vec3, Vec4 } from "../../../types";
import { BehaviorNode } from "./BehaviorNode";

export type BehaviorNodeParameters = Record<
  string,
  number | boolean | Vec2 | Vec3 | Vec4 | { $operation: BehaviorNode }
>;

export type BehaviorNodeParametersDef = Record<
  string,
  number | boolean | Vec2 | Vec3 | Vec4 | { $operation: number }
>;

export type BehaviorNodeExtras = {
  position?: { x: number; y: number };
  script?: string;
};
