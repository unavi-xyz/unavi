import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { VRM0Def, VRM0MetadataDef } from "./types";

interface IVRM0 extends IProperty, VRM0Def {}

/**
 * Root VRM 0.0 object.
 *
 * @group VRM 0.0
 * @see {@link VRM0MetadataExtension}
 */
export class VRM0 extends ExtensionProperty<IVRM0> {
  static override EXTENSION_NAME = EXTENSION_NAME.VRM0;
  declare extensionName: typeof EXTENSION_NAME.VRM0;
  declare propertyType: "VRM0";
  declare parentTypes: [PropertyType.ROOT];

  protected init() {
    this.extensionName = EXTENSION_NAME.VRM0;
    this.propertyType = "VRM0";
    this.parentTypes = [PropertyType.ROOT];
  }

  protected override getDefaults(): Nullable<IVRM0> {
    return Object.assign(super.getDefaults(), {
      exporterVersion: null,
      meta: {},
      humanoid: {},
      firstPerson: {},
      blendShapeMaster: {},
      secondaryAnimation: {},
      materialProperties: [],
    });
  }

  getMeta() {
    return this.get("meta");
  }

  setMeta(meta: VRM0MetadataDef) {
    return this.set("meta", meta);
  }
}
