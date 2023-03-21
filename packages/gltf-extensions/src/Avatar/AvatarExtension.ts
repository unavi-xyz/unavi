import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { Avatar } from "./Avatar";

type AvatarDef = {
  name: string;
  equippable: boolean;
  uri: string;
};

/**
 * Implementation of the WIRED_avatar extension.
 *
 * @group WIRED_avatar
 */
export class AvatarExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Avatar;
  override readonly extensionName = EXTENSION_NAME.Avatar;

  createAvatar(): Avatar {
    return new Avatar(this.document.getGraph());
  }

  read(context: ReaderContext) {
    const nodeDefs = context.jsonDoc.json.nodes || [];

    // Add avatars to nodes
    nodeDefs.forEach((nodeDef, nodeIndex) => {
      if (!nodeDef.extensions || !nodeDef.extensions[this.extensionName]) return;

      const node = context.nodes[nodeIndex];
      if (!node) throw new Error("Node not found");

      const avatar = this.createAvatar();

      const avatarDef = nodeDef.extensions[this.extensionName] as AvatarDef;
      avatar.setName(avatarDef.name);
      avatar.setEquippable(avatarDef.equippable);
      avatar.setURI(avatarDef.uri);

      node.setExtension(this.extensionName, avatar);
    });

    return this;
  }

  write(context: WriterContext) {
    this.document
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const avatar = node.getExtension<Avatar>(this.extensionName);
        if (!avatar) return;

        const nodeIndex = context.nodeIndexMap.get(node);
        if (nodeIndex === undefined) throw new Error("Node index not found");

        const nodes = context.jsonDoc.json.nodes;
        if (!nodes) throw new Error("Nodes not found");

        const nodeDef = nodes[nodeIndex];
        if (!nodeDef) throw new Error("Node def not found");

        nodeDef.extensions ??= {};

        const avatarDef: AvatarDef = {
          name: avatar.getName(),
          equippable: avatar.getEquippable(),
          uri: avatar.getURI(),
        };
        nodeDef.extensions[this.extensionName] = avatarDef;
      });

    return this;
  }
}
