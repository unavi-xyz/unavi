import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { SpaceDef, spaceSchema } from "./schemas";
import { Space } from "./Space";

/**
 * Implementation of the WIRED_space extension.
 *
 * @group WIRED_space
 */
export class SpaceExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Space;
  override readonly extensionName = EXTENSION_NAME.Space;

  public createSpace(): Space {
    return new Space(this.document.getGraph());
  }

  public read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = spaceSchema.safeParse(context.jsonDoc.json.extensions[this.extensionName]);

    if (!rootDef.success) {
      console.warn(rootDef.error);
      return this;
    }

    const space = this.createSpace();
    space.setHost(rootDef.data.host);

    if (rootDef.data.image) space.setImage(rootDef.data.image);

    this.document.getRoot().setExtension(this.extensionName, space);

    return this;
  }

  public write(context: WriterContext) {
    const space = this.document.getRoot().getExtension<Space>(this.extensionName);
    if (!space) return this;

    const spaceDef: SpaceDef = {
      host: space.getHost(),
      image: space.getImage(),
    };

    context.jsonDoc.json.extensions = context.jsonDoc.json.extensions || {};
    context.jsonDoc.json.extensions[this.extensionName] = spaceDef;

    return this;
  }
}
