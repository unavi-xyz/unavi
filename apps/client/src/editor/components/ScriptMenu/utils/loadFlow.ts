import {
  BehaviorNode,
  BehaviorNodeExtras,
  BehaviorNodeParametersJSON,
  Engine,
  isJsonPath,
  isLink,
  pathRefToString,
} from "engine";
import { nanoid } from "nanoid";
import { Edge, Node } from "reactflow";

import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Loads the engine nodes into reactflow
 */
export function loadFlow(engine: Engine, scriptId: string) {
  const nodes: Node<BehaviorNodeParametersJSON>[] = [];
  const edges: Edge[] = [];

  engine.scene.extensions.behavior.listProperties().forEach((property) => {
    if (!(property instanceof BehaviorNode)) return;

    const { name, type, parameters, flow } = property;
    const extras = property.getExtras() as BehaviorNodeExtras;

    if (extras.script !== scriptId) return;

    const data: BehaviorNodeParametersJSON = {};

    if (parameters) {
      Object.entries(parameters).forEach(([key, value]) => {
        if (isJsonPath(value) || isLink(value)) return;
        data[key] = value;
      });
    }

    nodes.push({ id: name, type, data, position: extras.position ?? { x: 0, y: 0 } });

    if (flow) {
      Object.entries(flow).forEach(([key, value]) => {
        edges.push({
          id: nanoid(),
          source: name,
          sourceHandle: key,
          target: value.name,
          targetHandle: "flow",
        });
      });
    }

    if (parameters) {
      Object.entries(parameters).forEach(([key, value]) => {
        if (!isLink(value)) return;

        edges.push({
          id: nanoid(),
          source: value.link.name,
          sourceHandle: value.socket,
          target: name,
          targetHandle: key,
        });
      });
    }
  });

  // Convert jsonPaths from ref to string
  nodes.forEach((node) => {
    const nodeSpec = nodeSpecJSON.find((nodeSpec) => nodeSpec.type === node.type);
    if (!nodeSpec) return;

    const behaviorNode = engine.scene.extensions.behavior
      .listProperties()
      .find((property): property is BehaviorNode => {
        if (!(property instanceof BehaviorNode)) return false;
        if (property.name !== node.id) return false;
        return true;
      });
    if (!behaviorNode) return;

    if (nodeSpec.inputs) {
      nodeSpec.inputs.forEach(({ name }) => {
        if (name !== "jsonPath") return;

        const value = behaviorNode.parameters?.[name];
        if (!isJsonPath(value)) return;

        const jsonPath = pathRefToString(value, engine);
        if (!jsonPath) return;

        node.data[name] = jsonPath;
      });
    }
  });

  return { nodes, edges };
}
