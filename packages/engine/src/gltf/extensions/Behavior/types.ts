import { Vec2, Vec3, Vec4 } from "../../../types";
import { BehaviorNode } from "./BehaviorNode";

export type BehaviorNodeParameters = Record<
  string,
  number | boolean | Vec2 | Vec3 | Vec4 | { link: BehaviorNode; socket: string }
>;

export type BehaviorNodeParametersDef = Record<
  string,
  number | boolean | Vec2 | Vec3 | Vec4 | { link: number; socket: string }
>;

export type BehaviorNodeExtras = {
  position?: { x: number; y: number };
  script?: string;
};
