import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";

interface IAvatar extends IProperty {
  uri: string;
}

/**
 * Represents a {@link https://vrm.dev/en/ VRM} avatar that the user can equip.
 *
 * @group WIRED_avatar_equip
 * @see {@link AvatarEquipExtension}
 */
export class Avatar extends ExtensionProperty<IAvatar> {
  static override EXTENSION_NAME = EXTENSION_NAME.AvatarEquip;
  declare extensionName: typeof EXTENSION_NAME.AvatarEquip;
  declare propertyType: "Avatar";
  declare parentTypes: [PropertyType.NODE];

  protected init() {
    this.extensionName = EXTENSION_NAME.AvatarEquip;
    this.propertyType = "Avatar";
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<IAvatar> {
    return Object.assign(super.getDefaults(), {
      uri: null,
    });
  }

  getURI(): string {
    return this.get("uri");
  }

  setURI(uri: string) {
    this.set("uri", uri);
    return this;
  }
}
