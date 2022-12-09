import { ExtensionProperty, IProperty, Mesh, Nullable, PropertyType } from "@gltf-transform/core";

import { Triplet } from "../../../types";
import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";
import { ColliderType, ICollider } from "./types";

export class Collider extends ExtensionProperty<ICollider> {
  static EXTENSION_NAME = EXTENSION_NAME;
  declare extensionName: typeof EXTENSION_NAME;
  declare propertyType: typeof PROPERTY_TYPE;
  declare parentTypes: [PropertyType.NODE];

  static Type: Record<string, ColliderType> = {
    MESH: "mesh",
    BOX: "box",
    SPHERE: "sphere",
    CYLINDER: "cylinder",
    CAPSULE: "capsule",
    HULL: "hull",
    COMPOUND: "compound",
  };

  protected init(): void {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [PropertyType.NODE];
  }

  protected getDefaults(): Nullable<ICollider> {
    return Object.assign(super.getDefaults() as IProperty, {
      type: null,
      size: null,
      radius: null,
      height: null,
      mesh: null,
    });
  }

  getType(): ColliderType {
    return this.get("type");
  }

  setType(type: ColliderType): this {
    return this.set("type", type);
  }

  getSize(): Triplet | null {
    return this.get("size");
  }

  setSize(size: Triplet | null): this {
    return this.set("size", size);
  }

  getRadius(): number | null {
    return this.get("radius");
  }

  setRadius(radius: number | null): this {
    return this.set("radius", radius);
  }

  getHeight(): number | null {
    return this.get("height");
  }

  setHeight(height: number | null): this {
    return this.set("height", height);
  }

  getMesh(): Mesh | null {
    return this.getRef("mesh");
  }

  setMesh(mesh: Mesh | null): this {
    return this.setRef("mesh", mesh);
  }
}
