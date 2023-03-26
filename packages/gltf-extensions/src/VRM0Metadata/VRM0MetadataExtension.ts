import { Extension, ReaderContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { VRM0Def, VRM0MetadataDef } from "./types";
import { VRM0 } from "./VRM0";

const defaultMeta: VRM0MetadataDef = {
  title: "",
};

/**
 * Partial implementation of the {@link https://github.com/vrm-c/vrm-specification/tree/master/specification/0.0 VRM 0.0} extension, to allow reading metadata.
 *
 * @group VRM 0.0
 */
export class VRM0MetadataExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.VRM0;
  override readonly extensionName = EXTENSION_NAME.VRM0;

  createVRM0() {
    return new VRM0(this.document.getGraph());
  }

  public read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = context.jsonDoc.json.extensions[this.extensionName] as VRM0Def;

    const vrm0 = this.createVRM0();
    const meta = Object.assign({}, defaultMeta, rootDef.meta);
    vrm0.setMeta(meta);

    this.document.getRoot().setExtension(this.extensionName, vrm0);

    return this;
  }

  public write() {
    return this;
  }
}
