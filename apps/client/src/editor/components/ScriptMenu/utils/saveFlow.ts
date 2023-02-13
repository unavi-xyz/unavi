import { BehaviorNode, BehaviorNodeExtras, Engine } from "engine";
import { Edge, Node } from "reactflow";

import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Saves reactflow nodes and edges into the engine
 */
export function saveFlow(nodes: Node[], edges: Edge[], engine: Engine) {
  // Add nodes to extension
  const behaviorNodes = nodes
    .map(({ id, type, data, position }) => {
      if (!type) return;

      const existingNode = engine.scene.extensions.behavior.listProperties().find((property) => {
        if (!(property instanceof BehaviorNode)) return false;
        if (property.name !== id) return false;
        return true;
      }) as BehaviorNode | undefined;

      const behaviorNode = existingNode ?? engine.scene.extensions.behavior.createBehaviorNode();

      behaviorNode.name = id;
      behaviorNode.type = type;
      behaviorNode.parameters = data;
      behaviorNode.flow = null;

      const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
      extras.position = position;

      return behaviorNode;
    })
    .filter((node): node is BehaviorNode => Boolean(node));

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
      const handle = sourceHandle === "flow" ? "next" : sourceHandle;
      sourceNode.flow[handle] = targetNode;
    } else {
      if (!targetNode.parameters) targetNode.parameters = {};
      targetNode.parameters[targetHandle] = { $operation: sourceNode };
    }
  });

  // Remove old nodes
  engine.scene.extensions.behavior.listProperties().forEach((property) => {
    if (!(property instanceof BehaviorNode)) return;
    if (behaviorNodes.find((node) => node.name === property.name)) return;

    property.dispose();
  });
}
