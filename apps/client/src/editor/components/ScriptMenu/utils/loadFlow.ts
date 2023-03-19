import { Engine, pathRefToString } from "engine";
import {
  BehaviorNode,
  BehaviorNodeExtras,
  isJsonPath,
  isLink,
  isVariableConfig,
} from "gltf-extensions";
import { nanoid } from "nanoid";
import { Edge, Node } from "reactflow";

import { FlowNodeData } from "../types";
import { getNodeSpecJSON } from "./getNodeSpecJSON";

const nodeSpecJSON = getNodeSpecJSON();

/**
 * Loads the engine nodes into reactflow
 */
export function loadFlow(engine: Engine, scriptId: string) {
  const nodes: Node<FlowNodeData>[] = [];
  const edges: Edge[] = [];

  const variables = engine.scene.extensions.behavior.listVariables();

  engine.scene.extensions.behavior.listBehaviorNodes().forEach((behaviorNode) => {
    const extras = behaviorNode.getExtras() as BehaviorNodeExtras;

    if (extras.script !== scriptId) return;

    const data: FlowNodeData = {};

    const config = behaviorNode.getConfiguration();

    if (config) {
      if (isVariableConfig(config)) {
        const variable = behaviorNode.getVariable();
        if (!variable) return;

        const variableIndex = variables.findIndex((v) => v === variable);

        data["variable"] = { variableId: variableIndex };
      }
    }

    const params = behaviorNode.getParameters();

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (isJsonPath(value) || isLink(value)) return;
        data[key] = value;
      });
    }

    nodes.push({
      id: behaviorNode.getName(),
      type: behaviorNode.getType(),
      data,
      position: extras.position ?? { x: 0, y: 0 },
    });

    if (behaviorNode.listFlowKeys().length > 0) {
      behaviorNode.listFlowKeys().forEach((key) => {
        const value = behaviorNode.getFlow(key);
        if (!value) return;

        edges.push({
          id: nanoid(),
          source: behaviorNode.getName(),
          sourceHandle: key,
          target: value.getName(),
          targetHandle: "flow",
        });
      });
    }

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (!isLink(value)) return;

        const linkedNode = behaviorNode.getLink(key);
        if (!linkedNode) return;

        edges.push({
          id: nanoid(),
          source: linkedNode.getName(),
          sourceHandle: value.link.socket,
          target: behaviorNode.getName(),
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
        if (property.getName() !== node.id) return false;
        return true;
      });
    if (!behaviorNode) return;

    if (nodeSpec.inputs) {
      nodeSpec.inputs.forEach(({ name }) => {
        if (name !== "jsonPath") return;

        const parameters = behaviorNode.getParameters();
        if (!parameters) return;

        const value = parameters[name];
        if (!isJsonPath(value)) return;

        const jsonNode = behaviorNode.getNode(name);
        if (!jsonNode) return;

        const jsonPath = pathRefToString(value, jsonNode, engine);
        if (!jsonPath) return;

        node.data[name] = { value: jsonPath };
      });
    }
  });

  return { nodes, edges };
}
