import {
  BehaviorNodeParameterValue,
  BehaviorNodeParameterValueJSON,
  JsonPathRef,
  ParameterLink,
  ParamLinkJSON,
} from "../gltf/extensions/Behavior/types";

export function isLink(param?: BehaviorNodeParameterValue): param is ParameterLink {
  return typeof param === "object" && "link" in param;
}

export function isLinkJSON(param?: BehaviorNodeParameterValueJSON): param is ParamLinkJSON {
  return typeof param === "object" && "link" in param;
}

export function isJsonPath(param?: BehaviorNodeParameterValue): param is JsonPathRef {
  return typeof param === "object" && "isJsonPath" in param && param.isJsonPath === true;
}
