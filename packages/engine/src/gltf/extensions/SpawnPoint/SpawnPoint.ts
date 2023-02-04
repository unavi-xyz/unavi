import { ExtensionProperty, IProperty, PropertyType } from "@gltf-transform/core";
import { Nullable } from "@gltf-transform/core/dist/constants";

import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";

interface ISpawnPoint extends IProperty {
  title: string;
}

export class SpawnPoint extends ExtensionProperty<ISpawnPoint> {
  static override EXTENSION_NAME = EXTENSION_NAME;
  declare extensionName: typeof EXTENSION_NAME;
  declare propertyType: typeof PROPERTY_TYPE;
  declare parentTypes: [PropertyType.NODE];

  protected init(): void {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<ISpawnPoint> {
    return Object.assign(super.getDefaults(), {
      title: null,
    });
  }

  get title(): string {
    return this.get("title");
  }

  set title(title: string) {
    this.set("title", title);
  }
}
