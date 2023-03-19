import { Engine } from "engine";
import {
  BehaviorNode,
  BehaviorNodeExtras,
  BehaviorNodeParameters,
  parseJSONPath,
} from "gltf-extensions";
import { Edge, Node as FlowNode } from "reactflow";

import { FlowNodeData } from "../types";
import { flowIsJsonPathJSON, flowIsLinkJSON, flowIsVariableJSON } from "./filters";
import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Saves reactflow nodes into the engine
 */
export function saveFlow(
  nodes: FlowNode<FlowNodeData>[],
  edges: Edge[],
  engine: Engine,
  scriptId: string
) {
  // Add behavior nodes to extension
  const behaviorNodes = nodes
    .map(({ id, type, data, position }) => {
      if (!type) return;

      const existingNode = engine.scene.extensions.behavior
        .listBehaviorNodes()
        .find((behaviorNode) => behaviorNode.getName() === id);

      const behaviorNode = existingNode ?? engine.scene.extensions.behavior.createBehaviorNode();

      const parameters: BehaviorNodeParameters = {};

      const variables = engine.scene.extensions.behavior.listVariables();

      Object.entries(data).forEach(([key, value]) => {
        if (flowIsLinkJSON(value)) return;

        if (flowIsVariableJSON(value)) {
          const variable = variables[value.variableId];
          if (variable) {
            behaviorNode.setConfiguration({ isVariable: true });
            behaviorNode.setVariable(variable);
          }
        } else {
          parameters[key] = value;
        }
      });

      behaviorNode.setName(id);
      behaviorNode.setType(type);
      behaviorNode.setParameters(parameters);

      const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
      extras.position = position;
      extras.script = scriptId;

      // Convert jsonPaths from string to ref
      const nodeSpec = nodeSpecJSON.find((nodeSpec) => nodeSpec.type === type);
      if (!nodeSpec) return;

      if (nodeSpec.inputs) {
        nodeSpec.inputs.forEach(({ name }) => {
          const value = data[name];
          if (!flowIsJsonPathJSON(name, value)) return;

          const path = parseJSONPath(value.value);
          if (!path) return;

          const { resource, index, property } = path;
          if (resource !== "nodes") return;

          const node = engine.scene.doc.getRoot().listNodes()[index];
          if (!node) return;

          const parameters = behaviorNode.getParameters() ?? {};
          parameters[name] = { isJsonPath: true, property };
          behaviorNode.setParameters(parameters);

          behaviorNode.setNode(name, node);
        });
      }

      return behaviorNode;
    })
    .filter((node): node is BehaviorNode => Boolean(node));

  edges.forEach(({ source, sourceHandle, target, targetHandle }) => {
    if (!sourceHandle || !targetHandle) return;

    const sourceNode = behaviorNodes.find((node) => node.getName() === source);
    const targetNode = behaviorNodes.find((node) => node.getName() === target);
    if (!sourceNode || !targetNode) return;

    const sourceSpec = nodeSpecJSON.find((nodeSpec) => nodeSpec.type === sourceNode.getType());
    if (!sourceSpec) return;

    const outputSpec = sourceSpec.outputs.find((output) => output.name === sourceHandle);
    if (!outputSpec) return;

    if (outputSpec.valueType === "flow") {
      sourceNode.setFlow(sourceHandle, targetNode);
    } else {
      const parameters = targetNode.getParameters() ?? {};
      parameters[targetHandle] = { link: { socket: sourceHandle } };
      targetNode.setParameters(parameters);

      targetNode.setLink(targetHandle, sourceNode);
    }
  });

  // Remove old behavior nodes
  engine.scene.extensions.behavior.listBehaviorNodes().forEach((behaviorNode) => {
    const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
    if (extras.script !== scriptId) return;

    if (behaviorNodes.includes(behaviorNode)) return;

    behaviorNode.dispose();
  });
}
