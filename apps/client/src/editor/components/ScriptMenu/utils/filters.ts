import { ConstantValue, ParamLinkJSON, Value } from "@wired-labs/gltf-extensions";

import { FlowNodeParamter, FlowVariableParamter } from "../types";

export function flowIsLinkJSON(param?: FlowNodeParamter): param is ParamLinkJSON {
  return typeof param === "object" && "link" in param;
}

export function flowIsVariableJSON(config?: FlowNodeParamter): config is FlowVariableParamter {
  return typeof config === "object" && "variableId" in config;
}

export function flowIsConstantJSON(config?: FlowNodeParamter): config is ConstantValue {
  return typeof config === "object" && "value" in config;
}

export function flowIsJsonPathJSON(
  key: string,
  config?: FlowNodeParamter
): config is Value<string> {
  return key === "jsonPath" && typeof config === "object" && "value" in config;
}
