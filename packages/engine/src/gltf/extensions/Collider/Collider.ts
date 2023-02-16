import { ExtensionProperty, IProperty, Mesh, Nullable, PropertyType } from "@gltf-transform/core";

import { Vec3 } from "../../../types";
import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";
import { ColliderType } from "./types";

interface ICollider extends IProperty {
  type: ColliderType;
  size: Vec3 | null;
  radius: number | null;
  height: number | null;
  mesh: Mesh;
}

export class Collider extends ExtensionProperty<ICollider> {
  static override EXTENSION_NAME = EXTENSION_NAME;
  declare extensionName: typeof EXTENSION_NAME;
  declare propertyType: typeof PROPERTY_TYPE;
  declare parentTypes: [PropertyType.NODE];

  static Type: Record<string, ColliderType> = {
    BOX: "box",
    SPHERE: "sphere",
    CYLINDER: "cylinder",
    CAPSULE: "capsule",
    HULL: "hull",
    TRIMESH: "trimesh",
  };

  protected init() {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<ICollider> {
    return Object.assign(super.getDefaults(), {
      type: null,
      size: null,
      radius: null,
      height: null,
      mesh: null,
    });
  }

  get type() {
    return this.get("type");
  }

  set type(type: ColliderType) {
    this.set("type", type);
  }

  get size() {
    return this.get("size");
  }

  set size(size: Vec3 | null) {
    this.set("size", size);
  }

  get radius() {
    return this.get("radius");
  }

  set radius(radius: number | null) {
    this.set("radius", radius);
  }

  get height() {
    return this.get("height");
  }

  set height(height: number | null) {
    this.set("height", height);
  }

  get mesh() {
    return this.getRef("mesh");
  }

  set mesh(mesh: Mesh | null) {
    this.setRef("mesh", mesh);
  }
}
