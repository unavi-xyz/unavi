import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";

interface IAvatar extends IProperty {
  equippable: boolean;
  uri: string;
}

/**
 * Represents a {@link https://vrm.dev/en/ VRM} avatar.
 *
 * @group WIRED_avatar
 * @see {@link AvatarExtension}
 */
export class Avatar extends ExtensionProperty<IAvatar> {
  static override EXTENSION_NAME = EXTENSION_NAME.Avatar;
  declare extensionName: typeof EXTENSION_NAME.Avatar;
  declare propertyType: "Avatar";
  declare parentTypes: [PropertyType.NODE];

  protected init() {
    this.extensionName = EXTENSION_NAME.Avatar;
    this.propertyType = "Avatar";
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<IAvatar> {
    return Object.assign(super.getDefaults(), {
      equippable: null,
      uri: null,
    });
  }

  getEquippable(): boolean {
    return this.get("equippable");
  }

  setEquippable(equippable: boolean) {
    this.set("equippable", equippable);
    return this;
  }

  getURI(): string {
    return this.get("uri");
  }

  setURI(uri: string) {
    this.set("uri", uri);
    return this;
  }
}
