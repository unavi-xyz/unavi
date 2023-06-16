import {
  ExtensionProperty,
  IProperty,
  Nullable,
  PropertyType,
} from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { VRMDef, VRMMetadataDef } from "./types";

interface IVRM extends IProperty, VRMDef {}

/**
 * Root VRM object.
 *
 * @group VRMC_vrm
 * @see {@link VRMMetadataExtension}
 */
export class VRM extends ExtensionProperty<IVRM> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.VRM;
  declare extensionName: typeof EXTENSION_NAME.VRM;
  declare propertyType: "VRM";
  declare parentTypes: [PropertyType.ROOT];

  protected init() {
    this.extensionName = EXTENSION_NAME.VRM;
    this.propertyType = "VRM";
    this.parentTypes = [PropertyType.ROOT];
  }

  protected override getDefaults(): Nullable<IVRM> {
    return Object.assign(super.getDefaults(), {
      expression: {},
      firstPerson: {},
      humanoid: {},
      lookAt: {},
      meta: {},
      specVersion: "1.0",
    });
  }

  getMeta() {
    return this.get("meta");
  }

  setMeta(meta: VRMMetadataDef) {
    return this.set("meta", meta);
  }
}
