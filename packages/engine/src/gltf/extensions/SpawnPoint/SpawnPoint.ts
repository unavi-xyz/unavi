import { ExtensionProperty, IProperty, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";

export class SpawnPoint extends ExtensionProperty<IProperty> {
  static override EXTENSION_NAME = EXTENSION_NAME;
  declare extensionName: typeof EXTENSION_NAME;
  declare propertyType: typeof PROPERTY_TYPE;
  declare parentTypes: [PropertyType.NODE];

  protected init(): void {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [PropertyType.NODE];
  }
}
