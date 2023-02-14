import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { BehaviorNode } from "./BehaviorNode";
import { EXTENSION_NAME } from "./constants";
import { BehaviorNodeExtras, BehaviorNodeParametersDef } from "./types";

type BehaviorNodeDef = {
  type: string;
  name: string;
  parameters?: BehaviorNodeParametersDef;
  flow?: Record<string, number>;
  extras?: BehaviorNodeExtras;
};

type BehaviorExtensionDef = {
  behaviorNodes: BehaviorNodeDef[];
};

/**
 * @link https://github.com/ux3d/glTF/tree/extensions/KHR_behavior/extensions/2.0/Khronos/KHR_behavior
 */
export class BehaviorExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME;
  override readonly extensionName = EXTENSION_NAME;

  createBehaviorNode(): BehaviorNode {
    return new BehaviorNode(this.document.getGraph());
  }

  read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[EXTENSION_NAME])
      return this;

    const rootDef = context.jsonDoc.json.extensions[EXTENSION_NAME] as BehaviorExtensionDef;

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

          if (typeof value === "object" && "$operation" in value) {
            const operationNode = behaviorNodes[value.$operation];
            if (!operationNode) throw new Error("Invalid behavior node reference");

            behaviorNode.parameters[key] = { $operation: operationNode };
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

          if (typeof value === "object" && "$operation" in value) {
            const operationIndex = behaviorNodes.indexOf(value.$operation);
            behaviorNodeDef.parameters[key] = { $operation: operationIndex };
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
      context.jsonDoc.json.extensions[EXTENSION_NAME] = rootDef;
    }

    return this;
  }
}
