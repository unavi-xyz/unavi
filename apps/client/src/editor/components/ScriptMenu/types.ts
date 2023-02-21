import { ParameterJSON } from "engine";

export type FlowVariableParamter = { variableId: number };
export type FlowNodeParamter = ParameterJSON | FlowVariableParamter;

export type FlowNodeData = Record<string, FlowNodeParamter>;
