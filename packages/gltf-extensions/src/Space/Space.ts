import { ExtensionProperty, IProperty, PropertyType } from "@gltf-transform/core";
import { Nullable } from "@gltf-transform/core/dist/constants";

import { EXTENSION_NAME } from "../constants";

interface ISpace extends IProperty {
  host: string;
  image: string;
}

/**
 * Represents a spawn point.
 *
 * @group OMI_spawn_point
 * @see {@link SpaceExtension}
 */
export class Space extends ExtensionProperty<ISpace> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Space;
  declare extensionName: typeof EXTENSION_NAME.Space;
  declare propertyType: "Space";
  declare parentTypes: [PropertyType.ROOT];

  protected init() {
    this.extensionName = EXTENSION_NAME.Space;
    this.propertyType = "Space";
    this.parentTypes = [PropertyType.ROOT];
  }

  protected override getDefaults(): Nullable<ISpace> {
    return Object.assign(super.getDefaults(), {
      host: "",
      image: "",
    });
  }

  getHost(): string {
    return this.get("host");
  }

  setHost(host: string): this {
    return this.set("host", host);
  }

  getImage(): string {
    return this.get("image");
  }

  setImage(image: string): this {
    return this.set("image", image);
  }
}
