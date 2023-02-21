import { GraphJSON, NodeJSON, VariableJSON } from "@behave-graph/core";
import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { parseJSONPath } from "../../../behavior";
import { EXTENSION_NAME } from "../constants";
import { BehaviorNode } from "./BehaviorNode";
import {
  isJsonPath,
  isJsonPathJSON,
  isLink,
  isLinkJSON,
  isVariableConfig,
  isVariableConfigJSON,
} from "./filters";
import {
  BehaviorNodeConfigurationJSON,
  BehaviorNodeExtras,
  BehaviorNodeParametersJSON,
} from "./types";
import { Variable } from "./Variable";

type BehaviorNodeDef = {
  configuration?: BehaviorNodeConfigurationJSON;
  extras?: BehaviorNodeExtras;
  flow?: Record<string, number>;
  name: string;
  parameters?: BehaviorNodeParametersJSON;
  type: string;
};

type VariableDef = {
  initialValue: any;
  name: string;
  type: string;
};

type BehaviorExtensionDef = {
  behaviorNodes: BehaviorNodeDef[];
  variables: VariableDef[];
};

export type BehaviorEvent = {
  type: "node-created" | "node-removed" | "variable-created" | "variable-removed";
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

  createVariable(): Variable {
    return new Variable(this.document.getGraph());
  }

  read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = context.jsonDoc.json.extensions[this.extensionName] as BehaviorExtensionDef;

    // Read variables
    const variables = rootDef.variables.map((variableDef) => {
      const variable = this.createVariable();
      variable.type = variableDef.type;
      variable.setName(variableDef.name);
      variable.initialValue = variableDef.initialValue;
      return variable;
    });

    // Read behavior nodes
    const behaviorNodes = rootDef.behaviorNodes.map((behaviorNodeDef) => {
      const behaviorNode = this.createBehaviorNode();
      behaviorNode.type = behaviorNodeDef.type;
      behaviorNode.setName(behaviorNodeDef.name);
      if (behaviorNodeDef.extras) behaviorNode.setExtras(behaviorNodeDef.extras);
      return behaviorNode;
    });

    rootDef.behaviorNodes.forEach(({ parameters, configuration, flow }, i) => {
      const behaviorNode = behaviorNodes[i];
      if (!behaviorNode) throw new Error("Behavior node not found");

      if (configuration) {
        if (isVariableConfigJSON(configuration)) {
          const variable = variables.find((_, i) => i === configuration.variableId);
          if (variable) {
            behaviorNode.configuration = { isVariable: true };
            behaviorNode.variable = variable;
          }
        } else {
          behaviorNode.configuration = configuration;
        }
      }

      if (parameters) {
        Object.entries(parameters).forEach(([key, value]) => {
          if (!behaviorNode.parameters) behaviorNode.parameters = {};

          if (isLinkJSON(value)) {
            const linkedNode = behaviorNodes[value.link.nodeId];
            if (!linkedNode) throw new Error("Invalid behavior node reference");

            behaviorNode.parameters[key] = {
              link: { socket: value.link.socket },
            };
            behaviorNode.setLink(key, linkedNode);
          } else if (isJsonPathJSON(key, value)) {
            const path = parseJSONPath(value.value);
            if (path) {
              const node = context.nodes[path.index];
              if (!node) throw new Error("Invalid behavior node reference");
              behaviorNode.parameters[key] = { isJsonPath: true, property: path.property };
              behaviorNode.setNode(key, node);
            }
          } else {
            behaviorNode.parameters[key] = value;
          }
        });
      }

      if (flow) {
        Object.entries(flow).forEach(([key, value]) => {
          const flowNode = behaviorNodes[value];
          if (!flowNode) throw new Error("Invalid behavior node reference");

          behaviorNode.setFlow(key, flowNode);
        });
      }
    });

    return this;
  }

  write(context: WriterContext) {
    // Write variables
    // Filter out unused variables
    const variables = this.listVariables().filter((variable) => variable.listParents().length > 0);

    const variableDefs: VariableDef[] = variables.map((variable) => ({
      type: variable.type,
      name: variable.getName(),
      initialValue: variable.initialValue,
    }));

    // Write behavior nodes
    const behaviorNodes = this.listBehaviorNodes();
    const behaviorNodeDefs: BehaviorNodeDef[] = behaviorNodes.map((behaviorNode) => {
      const extras = behaviorNode.getExtras() as BehaviorNodeExtras;
      return { type: behaviorNode.type, name: behaviorNode.getName(), extras };
    });

    behaviorNodeDefs.forEach((behaviorNodeDef, i) => {
      const behaviorNode = behaviorNodes[i];
      if (!behaviorNode) throw new Error("Invalid behavior node reference");

      if (behaviorNode.configuration) {
        if (isVariableConfig(behaviorNode.configuration)) {
          const variable = behaviorNode.variable;
          if (!variable) throw new Error("Invalid variable reference");

          const variableIndex = variables.indexOf(variable);
          if (variableIndex === -1) throw new Error("Invalid variable reference");

          behaviorNodeDef.configuration = { variableId: variableIndex };
        } else {
          behaviorNodeDef.configuration = behaviorNode.configuration;
        }
      }

      if (behaviorNode.parameters) {
        Object.entries(behaviorNode.parameters).forEach(([key, value]) => {
          if (!behaviorNodeDef.parameters) behaviorNodeDef.parameters = {};

          if (isLink(value)) {
            const linkedNode = behaviorNode.getLink(key);
            if (!linkedNode) throw new Error("Invalid behavior node reference");

            const linkIndex = behaviorNodes.indexOf(linkedNode);
            if (linkIndex === -1) throw new Error("Invalid behavior node reference");

            behaviorNodeDef.parameters[key] = {
              link: { nodeId: linkIndex, socket: value.link.socket },
            };
          } else if (isJsonPath(value)) {
            const jsonNode = behaviorNode.getNode(key);
            if (!jsonNode) throw new Error("Invalid behavior node reference");

            const index = context.nodeIndexMap.get(jsonNode);
            if (index === undefined) throw new Error("Invalid node reference");
            behaviorNodeDef.parameters[key] = { value: `/nodes/${index}/${value.property}` };
          } else {
            behaviorNodeDef.parameters[key] = value;
          }
        });
      }

      if (behaviorNode.listFlowKeys().length > 0) {
        behaviorNode.listFlowKeys().forEach((key) => {
          const value = behaviorNode.getFlow(key);
          if (!value) throw new Error("Invalid behavior node reference");

          if (!behaviorNodeDef.flow) behaviorNodeDef.flow = {};

          const flowIndex = behaviorNodes.indexOf(value);
          if (flowIndex === -1) throw new Error("Invalid behavior node reference");

          behaviorNodeDef.flow[key] = flowIndex;
        });
      }
    });

    if (behaviorNodeDefs.length > 0 || variableDefs.length > 0) {
      const rootDef: BehaviorExtensionDef = {
        behaviorNodes: behaviorNodeDefs,
        variables: variableDefs,
      };

      if (!context.jsonDoc.json.extensions) context.jsonDoc.json.extensions = {};
      context.jsonDoc.json.extensions[this.extensionName] = rootDef;
    }

    return this;
  }

  /**
   * Exports the graph to a JSON object, which can be loaded into `behave-graph`.
   * @returns The exported JSON object.
   */
  toGraphJSON(): GraphJSON {
    const variables = this.listVariables();
    const behaviorNodes = this.listBehaviorNodes();

    const variableJSON: VariableJSON[] = variables.map((variable, i) => ({
      id: String(i),
      name: variable.getName(),
      label: variable.getName(),
      valueTypeName: variable.type,
      initialValue: variable.initialValue,
    }));

    const nodes: NodeJSON[] = behaviorNodes.map((behaviorNode, i) => {
      return {
        id: String(i),
        label: behaviorNode.getName(),
        type: behaviorNode.type,
      };
    });

    behaviorNodes.forEach((behaviorNode, i) => {
      const node = nodes[i];
      if (!node) throw new Error("Node not found");

      if (behaviorNode.listFlowKeys().length > 0) {
        behaviorNode.listFlowKeys().forEach((key) => {
          const value = behaviorNode.getFlow(key);
          if (!value) throw new Error("Invalid behavior node reference");

          const targetNodeIndex = behaviorNodes.indexOf(value);
          if (targetNodeIndex === -1) throw new Error("Invalid behavior node reference");

          const targetNode = nodes[targetNodeIndex];
          if (!targetNode) throw new Error("Invalid behavior node reference");

          if (!node.flows) node.flows = {};
          node.flows[key] = { nodeId: targetNode.id, socket: "flow" };
        });
      }

      if (behaviorNode.configuration) {
        if (isVariableConfig(behaviorNode.configuration)) {
          const variable = behaviorNode.variable;
          if (!variable) throw new Error("Invalid variable reference");

          const variableIndex = variables.indexOf(variable);
          if (variableIndex === -1) throw new Error("Invalid variable reference");

          node.configuration = { variableId: variableIndex };
        } else {
          node.configuration = behaviorNode.configuration;
        }
      }

      if (behaviorNode.parameters) {
        Object.entries(behaviorNode.parameters).forEach(([key, value]) => {
          if (!node.parameters) node.parameters = {};

          if (isLink(value)) {
            const linkedNode = behaviorNode.getLink(key);
            if (!linkedNode) throw new Error("Invalid behavior node reference");

            const targetNodeIndex = behaviorNodes.indexOf(linkedNode);
            if (targetNodeIndex === -1) throw new Error("Invalid behavior node reference");

            const targetNode = nodes[targetNodeIndex];
            if (!targetNode) throw new Error("Invalid behavior node reference");

            node.parameters[key] = { link: { nodeId: targetNode.id, socket: value.link.socket } };
          } else if (isJsonPath(value)) {
            const nodes = this.document.getRoot().listNodes();

            const jsonNode = behaviorNode.getNode(key);
            if (!jsonNode) throw new Error("Invalid behavior node reference");

            const index = nodes.indexOf(jsonNode);
            if (index === -1) throw new Error("Invalid node reference");

            node.parameters[key] = { value: `/nodes/${index}/${value.property}` };
          } else {
            node.parameters[key] = value;
          }
        });
      }
    });

    return { nodes, variables: variableJSON };
  }

  listBehaviorNodes(): BehaviorNode[] {
    return this.listProperties().filter(
      (property): property is BehaviorNode => property instanceof BehaviorNode
    );
  }

  listVariables(): Variable[] {
    return this.listProperties().filter(
      (property): property is Variable => property instanceof Variable
    );
  }
}
