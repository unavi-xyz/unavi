import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { Vec2, Vec3, Vec4 } from "../../../types";
import { BehaviorNode } from "./BehaviorNode";
import { EXTENSION_NAME } from "./constants";

type BehaviorNodeDef = {
  type: string;
  name: string;
  parameters?: Record<string, number | boolean | Vec2 | Vec3 | Vec4>;
  flow?: Record<string, number>;
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

  read(context: ReaderContext): this {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[EXTENSION_NAME])
      return this;

    const rootDef = context.jsonDoc.json.extensions[EXTENSION_NAME] as BehaviorExtensionDef;

    rootDef.behaviorNodes.forEach((behaviorNodeDef) => {
      const behaviorNode = this.createBehaviorNode();
      behaviorNode.type = behaviorNodeDef.type;
      behaviorNode.name = behaviorNodeDef.name;
      behaviorNode.parameters = behaviorNodeDef.parameters ?? null;
      behaviorNode.flow = behaviorNodeDef.flow ?? null;
    });

    return this;
  }

  write(context: WriterContext): this {
    const behaviorNodeDefs = [];

    for (const property of this.properties) {
      if (property instanceof BehaviorNode) {
        const behaviorNode = property as BehaviorNode;

        const behaviorNodeDef: BehaviorNodeDef = {
          type: behaviorNode.type,
          name: behaviorNode.name,
          parameters: behaviorNode.parameters ?? undefined,
          flow: behaviorNode.flow ?? undefined,
        };

        behaviorNodeDefs.push(behaviorNodeDef);
      }
    }

    if (behaviorNodeDefs.length > 0) {
      const rootDef: BehaviorExtensionDef = { behaviorNodes: behaviorNodeDefs };

      if (!context.jsonDoc.json.extensions) context.jsonDoc.json.extensions = {};
      context.jsonDoc.json.extensions[EXTENSION_NAME] = rootDef;
    }

    return this;
  }
}
