import {
  BehaviorNodeConfiguration,
  BehaviorNodeConfigurationJSON,
  Parameter,
  ParameterJSON,
  ParamJsonPath,
  ParamLink,
  ParamLinkJSON,
  Value,
  VariableConfig,
  VariableConfigJSON,
} from "../types";

// Parameter
export function isLink(param?: Parameter): param is ParamLink {
  return typeof param === "object" && "link" in param;
}

export function isLinkJSON(param?: ParameterJSON): param is ParamLinkJSON {
  return typeof param === "object" && "link" in param;
}

export function isJsonPath(param?: Parameter): param is ParamJsonPath {
  return typeof param === "object" && "isJsonPath" in param && param.isJsonPath === true;
}

export function isJsonPathJSON(key: string, param?: ParameterJSON): param is Value<string> {
  return (
    key === "jsonPath" &&
    typeof param === "object" &&
    "value" in param &&
    typeof param.value === "string"
  );
}

// Configuration
export function isVariableConfig(config?: BehaviorNodeConfiguration): config is VariableConfig {
  return typeof config === "object" && "isVariable" in config;
}

export function isVariableConfigJSON(
  config?: BehaviorNodeConfigurationJSON
): config is VariableConfigJSON {
  return typeof config === "object" && "variableId" in config;
}
