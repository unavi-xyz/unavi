import { ParameterJSON } from "@wired-labs/gltf-extensions";

export type FlowVariableParamter = { variableId: number };
export type FlowNodeParamter = ParameterJSON | FlowVariableParamter;

export type FlowNodeData = Record<string, FlowNodeParamter>;
