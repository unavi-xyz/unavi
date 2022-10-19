import {
  ExtensionProperty,
  IProperty,
  PropertyType,
} from "@gltf-transform/core";

import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";

export class SpawnPoint extends ExtensionProperty<IProperty> {
  public static readonly EXTENSION_NAME = EXTENSION_NAME;
  public declare extensionName: typeof EXTENSION_NAME;
  public declare propertyType: typeof PROPERTY_TYPE;
  public declare parentTypes: [PropertyType.NODE];

  protected init(): void {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [PropertyType.NODE];
  }
}
