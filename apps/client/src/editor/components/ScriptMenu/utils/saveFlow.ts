import { BehaviorNode, BehaviorNodeExtras, Engine } from "engine";
import { Edge, Node as FlowNode } from "reactflow";

import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Saves reactflow nodes into the engine
 */
export function saveFlow(nodes: FlowNode[], edges: Edge[], engine: Engine, scriptId: string) {
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

      behaviorNode.name = id;
      behaviorNode.type = type;
      behaviorNode.parameters = data;
      behaviorNode.flow = null;

      const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
      extras.position = position;
      extras.script = scriptId;

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
