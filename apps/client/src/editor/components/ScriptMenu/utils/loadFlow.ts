import { BehaviorNode, BehaviorNodeExtras, Engine } from "engine";
import { nanoid } from "nanoid";
import { Edge, Node } from "reactflow";

/**
 * Loads the engine nodes into reactflow
 */
export function loadFlow(engine: Engine) {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  engine.scene.extensions.behavior.listProperties().forEach((property) => {
    if (!(property instanceof BehaviorNode)) return;

    const { name, type, parameters, flow } = property;
    const extras = property.getExtras() as BehaviorNodeExtras;

    nodes.push({
      id: name,
      type,
      data: parameters ?? {},
      position: extras.position ?? { x: 0, y: 0 },
    });

    if (flow) {
      Object.entries(flow).forEach(([key, value]) => {
        edges.push({
          id: nanoid(),
          source: name,
          sourceHandle: key === "next" ? "flow" : key,
          target: value.name,
          targetHandle: "flow",
        });
      });
    }

    if (parameters) {
      Object.entries(parameters).forEach(([key, value]) => {
        if (typeof value !== "object" || !("$operation" in value)) return;
        const operationNode = value.$operation;
        edges.push({
          id: nanoid(),
          source: operationNode.name,
          sourceHandle: "flow",
          target: name,
          targetHandle: key,
        });
      });
    }
  });

  return { nodes, edges };
}
