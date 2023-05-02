import { Extension, ReaderContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { VRMDef, VRMMetadataDef } from "./types";
import { VRM } from "./VRM";

const defaultMeta: VRMMetadataDef = {
  allowAntisocialOrHateUsage: false,
  allowExcessivelySexualUsage: false,
  allowExcessivelyViolentUsage: false,
  allowPoliticalOrReligiousUsage: false,
  allowRedistribution: false,
  authors: [],
  avatarPermission: "onlyAuthor",
  commercialUsage: "personalNonProfit",
  creditNotation: "required",
  licenseUrl: "",
  modification: "prohibited",
  name: "",
};

/**
 * Partial implementation of the {@link https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm-1.0 VRMC_vrm} extension, to allow reading metadata.
 *
 * @group VRMC_vrm
 */
export class VRMMetadataExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.VRM;
  override readonly extensionName = EXTENSION_NAME.VRM;

  createVRM() {
    return new VRM(this.document.getGraph());
  }

  public read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = context.jsonDoc.json.extensions[this.extensionName] as VRMDef;

    const vrm = this.createVRM();
    const meta = Object.assign({}, defaultMeta, rootDef.meta);
    vrm.setMeta(meta);

    this.document.getRoot().setExtension(this.extensionName, vrm);

    return this;
  }

  public write() {
    return this;
  }
}
