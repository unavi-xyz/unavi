import {
  ExtensionProperty,
  IProperty,
  Mesh,
  Nullable,
  PropertyType,
} from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { Vec3 } from "../types";
import { ColliderType } from "./schemas";

interface ICollider extends IProperty {
  type: ColliderType;
  isTrigger: boolean;
  size: Vec3 | null;
  radius: number | null;
  height: number | null;
  mesh: Mesh;
}

/**
 * Represents a physics collider.
 *
 * @group OMI_collider
 * @see {@link ColliderExtension}
 */
export class Collider extends ExtensionProperty<ICollider> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Collider;
  declare extensionName: typeof EXTENSION_NAME.Collider;
  declare propertyType: "Collider";
  declare parentTypes: [PropertyType.NODE];

  static Type: Record<string, ColliderType> = {
    BOX: "box",
    CAPSULE: "capsule",
    CYLINDER: "cylinder",
    HULL: "hull",
    SPHERE: "sphere",
    TRIMESH: "trimesh",
  };

  protected init() {
    this.extensionName = EXTENSION_NAME.Collider;
    this.propertyType = "Collider";
    this.parentTypes = [PropertyType.NODE];
  }

  protected override getDefaults(): Nullable<ICollider> {
    return Object.assign(super.getDefaults(), {
      height: null,
      isTrigger: false,
      mesh: null,
      radius: null,
      size: null,
      type: null,
    });
  }

  getType() {
    return this.get("type");
  }

  setType(type: ColliderType) {
    this.set("type", type);
  }

  getIsTrigger() {
    return this.get("isTrigger");
  }

  setIsTrigger(isTrigger: boolean) {
    this.set("isTrigger", isTrigger);
  }

  getSize() {
    return this.get("size");
  }

  setSize(size: Vec3 | null) {
    this.set("size", size);
  }

  getRadius() {
    return this.get("radius");
  }

  setRadius(radius: number | null) {
    this.set("radius", radius);
  }

  getHeight() {
    return this.get("height");
  }

  setHeight(height: number | null) {
    this.set("height", height);
  }

  getMesh(): Mesh | null {
    return this.getRef("mesh");
  }

  setMesh(mesh: Mesh | null) {
    this.setRef("mesh", mesh);
  }
}
