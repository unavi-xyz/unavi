import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { Avatar } from "./Avatar";
import { AvatarDef, avatarSchema } from "./schemas";

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
      if (!node) throw new Error(`Node ${nodeIndex} not found.`);

      const avatar = this.createAvatar();

      const parsedAvatarDef = avatarSchema.safeParse(nodeDef.extensions[this.extensionName]);

      if (!parsedAvatarDef.success) {
        console.warn(parsedAvatarDef.error);
        return;
      }

      const avatarDef = parsedAvatarDef.data;

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
        if (nodeIndex === undefined) return;

        const nodes = context.jsonDoc.json.nodes;
        if (!nodes) return;

        const nodeDef = nodes[nodeIndex];
        if (!nodeDef) return;

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
