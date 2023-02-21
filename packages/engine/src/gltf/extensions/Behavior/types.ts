import { Node } from "@gltf-transform/core";

import { Vec2, Vec3, Vec4 } from "../../../types";
import { BehaviorNode } from "./BehaviorNode";

export type ParameterLink = { link: BehaviorNode; socket: string };
export type JsonPathRef = { isJsonPath: true; node: Node; property: string };
export type BehaviorNodeParameterValue =
  | string
  | number
  | boolean
  | Vec2
  | Vec3
  | Vec4
  | ParameterLink
  | JsonPathRef;

export type BehaviorNodeParameters = Record<string, BehaviorNodeParameterValue>;

export type ParamLinkJSON = { link: number; socket: string };
export type BehaviorNodeParameterValueJSON =
  | string
  | number
  | boolean
  | Vec2
  | Vec3
  | Vec4
  | ParamLinkJSON;

export type BehaviorNodeParametersJSON = Record<string, BehaviorNodeParameterValueJSON>;

export type BehaviorNodeExtras = {
  position?: { x: number; y: number };
  script?: string;
};

export const ValueType = {
  boolean: "boolean",
  float: "float",
  flow: "flow",
  integer: "integer",
  number: "number",
  string: "string",
  vec2: "vec2",
  vec3: "vec3",
  vec4: "vec4",
  euler: "euler",
  quat: "quat",
} as const;
