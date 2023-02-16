import {
  BehaviorNode,
  BehaviorNodeExtras,
  BehaviorNodeParameters,
  BehaviorNodeParametersJSON,
  Engine,
  isLinkJSON,
  parseJSONPath,
} from "engine";
import { Edge, Node as FlowNode } from "reactflow";

import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Saves reactflow nodes into the engine
 */
export function saveFlow(
  nodes: FlowNode<BehaviorNodeParametersJSON>[],
  edges: Edge[],
  engine: Engine,
  scriptId: string
) {
  // Add behavior nodes to extension
  const behaviorNodes = nodes
    .map(({ id, type, data, position }) => {
      if (!type) return;

      const existingNode = engine.scene.extensions.behavior.listProperties().find((property) => {
        if (!(property instanceof BehaviorNode)) return false;
        if (property.name !== id) return false;
        return true;
      }) as BehaviorNode | undefined;

      const behaviorNode = existingNode ?? engine.scene.extensions.behavior.createBehaviorNode();

      const parameters: BehaviorNodeParameters = {};

      Object.entries(data).forEach(([key, value]) => {
        if (isLinkJSON(value)) return;
        parameters[key] = value;
      });

      behaviorNode.name = id;
      behaviorNode.type = type;
      behaviorNode.parameters = parameters;
      behaviorNode.flow = null;

      const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
      extras.position = position;
      extras.script = scriptId;

      return behaviorNode;
    })
    .filter((node): node is BehaviorNode => Boolean(node));

  behaviorNodes.forEach((behaviorNode) => {
    const nodeSpec = nodeSpecJSON.find((nodeSpec) => nodeSpec.type === behaviorNode.type);
    if (!nodeSpec) return;

    // Convert jsonPaths from string to ref
    if (nodeSpec.inputs) {
      nodeSpec.inputs.forEach(({ name }) => {
        if (name !== "jsonPath" || !behaviorNode.parameters) return;

        const value = behaviorNode.parameters[name];
        if (!(typeof value === "string")) return;

        const path = parseJSONPath(value);
        if (!path) return;

        const { resource, index, property } = path;
        if (resource !== "nodes") return;

        const node = engine.scene.doc.getRoot().listNodes()[index];
        if (!node) return;

        behaviorNode.parameters[name] = { isJsonPath: true, node, property };
      });
    }
  });

  edges.forEach(({ source, sourceHandle, target, targetHandle }) => {
    if (!sourceHandle || !targetHandle) return;

    const sourceNode = behaviorNodes.find((node) => node.name === source);
    const targetNode = behaviorNodes.find((node) => node.name === target);
    if (!sourceNode || !targetNode) return;

    const sourceSpec = nodeSpecJSON.find((nodeSpec) => nodeSpec.type === sourceNode.type);
    if (!sourceSpec) return;

    const outputSpec = sourceSpec.outputs.find((output) => output.name === sourceHandle);
    if (!outputSpec) return;

    if (outputSpec.valueType === "flow") {
      if (!sourceNode.flow) sourceNode.flow = {};
      sourceNode.flow[sourceHandle] = targetNode;
    } else {
      if (!targetNode.parameters) targetNode.parameters = {};
      targetNode.parameters[targetHandle] = { link: sourceNode, socket: sourceHandle };
    }
  });

  // Remove old behavior nodes
  engine.scene.extensions.behavior.listProperties().forEach((behaviorNode) => {
    if (!(behaviorNode instanceof BehaviorNode)) return;

    const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
    if (extras.script !== scriptId) return;

    if (behaviorNodes.includes(behaviorNode)) return;

    behaviorNode.dispose();
  });
}
