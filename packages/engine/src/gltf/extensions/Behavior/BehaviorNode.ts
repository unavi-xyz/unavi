import { ExtensionProperty, IProperty } from "@gltf-transform/core";

import { Vec2, Vec3, Vec4 } from "../../../types";
import { EXTENSION_NAME, PROPERTY_TYPE } from "./constants";

interface IBehaviorNode extends IProperty {
  type: string;
  name: string;
  parameters: Record<string, number | boolean | Vec2 | Vec3 | Vec4> | null;
  flow: Record<string, number> | null;
}

export class BehaviorNode extends ExtensionProperty<IBehaviorNode> {
  static override EXTENSION_NAME = EXTENSION_NAME;
  declare extensionName: typeof EXTENSION_NAME;
  declare propertyType: typeof PROPERTY_TYPE;
  declare parentTypes: [];

  protected init() {
    this.extensionName = EXTENSION_NAME;
    this.propertyType = PROPERTY_TYPE;
    this.parentTypes = [];
  }

  get type() {
    return this.get("type");
  }

  set type(type: string) {
    this.set("type", type);
  }

  get name() {
    return this.get("name");
  }

  set name(name: string) {
    this.set("name", name);
  }

  get parameters() {
    return this.get("parameters");
  }

  set parameters(parameters: Record<string, number | boolean | Vec2 | Vec3 | Vec4> | null) {
    this.set("parameters", parameters);
  }

  get flow() {
    return this.get("flow");
  }

  set flow(flow: Record<string, number> | null) {
    this.set("flow", flow);
  }
}
