import {
  NodeSpecJSON,
  registerCoreProfile,
  registerSceneProfile,
  Registry,
  writeNodeSpecsToJSON,
} from "@behave-graph/core";

let nodeSpecJSON: NodeSpecJSON[] | undefined = undefined;

export function getNodeSpecJSON() {
  if (nodeSpecJSON === undefined) {
    const registry = new Registry();
    registerCoreProfile(registry);
    registerSceneProfile(registry);
    nodeSpecJSON = writeNodeSpecsToJSON(registry);
  }

  return nodeSpecJSON;
}
