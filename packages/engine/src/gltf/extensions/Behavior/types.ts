import { Vec2, Vec3, Vec4 } from "../../../types";

export type ParamJsonPath = { isJsonPath: true; property: string };

export type ParamLink = { link: { socket: string } };
export type ParamLinkJSON = { link: { nodeId: number; socket: string } };

export type Value<T = any> = { value: T };
export type ConstantValue =
  | Value<string>
  | Value<number>
  | Value<boolean>
  | Value<Vec2>
  | Value<Vec3>
  | Value<Vec4>;

export type Parameter = ConstantValue | ParamLink | ParamJsonPath;
export type ParameterJSON = ConstantValue | ParamLinkJSON;

export type BehaviorNodeParameters = Record<string, Parameter>;
export type BehaviorNodeParametersJSON = Record<string, ParameterJSON>;

export type VariableConfig = { isVariable: true };
export type VariableConfigJSON = { variableId: number };

export type CustomEventConfig = { custromEventId: number };
export type FlowSequenceConfig = { numOutputs: number };
export type FlowSwitchConfig = { numCases: number };
export type FlowWaitAllConfig = { numInputs: number };

export type BehaviorNodeConfiguration =
  | VariableConfig
  | CustomEventConfig
  | FlowSequenceConfig
  | FlowSwitchConfig
  | FlowWaitAllConfig;
export type BehaviorNodeConfigurationJSON =
  | VariableConfigJSON
  | CustomEventConfig
  | FlowSequenceConfig
  | FlowSwitchConfig
  | FlowWaitAllConfig;

export type BehaviorNodeExtras = {
  position?: { x: number; y: number };
  script?: string;
};

export const ValueType = {
  boolean: "boolean",
  euler: "euler",
  float: "float",
  flow: "flow",
  integer: "integer",
  mat3: "mat3",
  mat4: "mat4",
  number: "number",
  quat: "quat",
  string: "string",
  vec2: "vec2",
  vec3: "vec3",
  vec4: "vec4",
} as const;
