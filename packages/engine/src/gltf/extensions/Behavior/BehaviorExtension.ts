import { GraphJSON, NodeJSON } from "@behave-graph/core";
import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { isJsonPath, isLink } from "../../../behavior";
import { EXTENSION_NAME } from "../constants";
import { BehaviorNode } from "./BehaviorNode";
import { BehaviorNodeExtras, BehaviorNodeParametersJSON } from "./types";

type BehaviorNodeDef = {
  type: string;
  name: string;
  parameters?: BehaviorNodeParametersJSON;
  flow?: Record<string, number>;
  extras?: BehaviorNodeExtras;
};

type BehaviorExtensionDef = {
  behaviorNodes: BehaviorNodeDef[];
};

/**
 * Implementation of the {@link https://github.com/ux3d/glTF/tree/extensions/KHR_behavior/extensions/2.0/Khronos/KHR_behavior KHR_behavior} extension.
 *
 * @group GLTF Extensions
 */
export class BehaviorExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Behavior;
  override readonly extensionName = EXTENSION_NAME.Behavior;

  createBehaviorNode(): BehaviorNode {
    return new BehaviorNode(this.document.getGraph());
  }

  read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = context.jsonDoc.json.extensions[this.extensionName] as BehaviorExtensionDef;

    const behaviorNodes = rootDef.behaviorNodes.map((behaviorNodeDef) => {
      const behaviorNode = this.createBehaviorNode();
      behaviorNode.type = behaviorNodeDef.type;
      behaviorNode.name = behaviorNodeDef.name;
      if (behaviorNodeDef.extras) behaviorNode.setExtras(behaviorNodeDef.extras);
      return behaviorNode;
    });

    rootDef.behaviorNodes.forEach(({ parameters, flow }, i) => {
      const behaviorNode = behaviorNodes[i];
      if (!behaviorNode) throw new Error("Behavior node not found");

      if (parameters) {
        Object.entries(parameters).forEach(([key, value]) => {
          if (!behaviorNode.parameters) behaviorNode.parameters = {};

          if (typeof value === "object" && "link" in value) {
            const operationNode = behaviorNodes[value.link];
            if (!operationNode) throw new Error("Invalid behavior node reference");

            behaviorNode.parameters[key] = { link: operationNode, socket: value.socket };
          } else {
            behaviorNode.parameters[key] = value;
          }
        });
      }

      if (flow) {
        Object.entries(flow).forEach(([key, value]) => {
          const flowNode = behaviorNodes[value];
          if (!flowNode) throw new Error("Invalid behavior node reference");

          if (!behaviorNode.flow) behaviorNode.flow = {};
          behaviorNode.flow[key] = flowNode;
        });
      }
    });

    return this;
  }

  write(context: WriterContext) {
    const behaviorNodes: BehaviorNode[] = [];
    const behaviorNodeDefs: BehaviorNodeDef[] = [];

    this.listProperties().forEach((property) => {
      if (!(property instanceof BehaviorNode)) return;

      const extras = property.getExtras() as BehaviorNodeExtras;

      behaviorNodeDefs.push({ type: property.type, name: property.name, extras });
      behaviorNodes.push(property);
    });

    behaviorNodeDefs.forEach((behaviorNodeDef, i) => {
      const behaviorNode = behaviorNodes[i];
      if (!behaviorNode) throw new Error("Invalid behavior node reference");

      if (behaviorNode.parameters) {
        Object.entries(behaviorNode.parameters).forEach(([key, value]) => {
          if (!behaviorNodeDef.parameters) behaviorNodeDef.parameters = {};

          if (isLink(value)) {
            const linkIndex = behaviorNodes.indexOf(value.link);
            behaviorNodeDef.parameters[key] = { link: linkIndex, socket: value.socket };
          } else if (isJsonPath(value)) {
            const index = context.nodeIndexMap.get(value.node);
            if (index === undefined) throw new Error("Invalid node reference");
            behaviorNodeDef.parameters[key] = `/nodes/${index}/${value.property}`;
          } else {
            behaviorNodeDef.parameters[key] = value;
          }
        });
      }

      if (behaviorNode.flow) {
        Object.entries(behaviorNode.flow).forEach(([key, value]) => {
          if (!behaviorNodeDef.flow) behaviorNodeDef.flow = {};

          const flowIndex = behaviorNodes.indexOf(value);
          behaviorNodeDef.flow[key] = flowIndex;
        });
      }
    });

    if (behaviorNodeDefs.length > 0) {
      const rootDef: BehaviorExtensionDef = { behaviorNodes: behaviorNodeDefs };

      if (!context.jsonDoc.json.extensions) context.jsonDoc.json.extensions = {};
      context.jsonDoc.json.extensions[this.extensionName] = rootDef;
    }

    return this;
  }

  toJSON(): GraphJSON {
    const behaviorNodes = this.listProperties().filter(
      (property): property is BehaviorNode => property instanceof BehaviorNode
    );

    const nodes: NodeJSON[] = behaviorNodes.map((behaviorNode, i) => {
      return {
        id: String(i),
        label: behaviorNode.name,
        type: behaviorNode.type,
      };
    });

    behaviorNodes.forEach((behaviorNode, i) => {
      const node = nodes[i];
      if (!node) throw new Error("Node not found");

      if (behaviorNode.flow) {
        Object.entries(behaviorNode.flow).forEach(([key, value]) => {
          const targetNodeIndex = behaviorNodes.indexOf(value);
          const targetNode = nodes[targetNodeIndex];
          if (!targetNode) throw new Error("Invalid behavior node reference");

          if (!node.flows) node.flows = {};
          node.flows[key] = { nodeId: targetNode.id, socket: "flow" };
        });
      }

      if (behaviorNode.parameters) {
        Object.entries(behaviorNode.parameters).forEach(([key, value]) => {
          if (!node.parameters) node.parameters = {};

          if (isLink(value)) {
            const targetNodeIndex = behaviorNodes.indexOf(value.link);
            const targetNode = nodes[targetNodeIndex];
            if (!targetNode) throw new Error("Invalid behavior node reference");

            node.parameters[key] = { link: { nodeId: targetNode.id, socket: value.socket } };
          } else if (isJsonPath(value)) {
            const nodes = this.document.getRoot().listNodes();
            const index = nodes.indexOf(value.node);
            node.parameters[key] = { value: `/nodes/${index}/${value.property}` };
          } else {
            node.parameters[key] = { value };
          }
        });
      }
    });

    return { nodes };
  }
}
