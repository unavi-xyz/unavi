import { ExtensionProperty, IProperty, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { BehaviorNodeParameters } from "./types";

interface IBehaviorNode extends IProperty {
  type: string;
  name: string;
  parameters: BehaviorNodeParameters | null;
  flow: Record<string, BehaviorNode> | null;
}

/**
 * Represents a single node in the behavior graph.
 *
 * @group GLTF Extensions
 * @see {@link BehaviorExtension}
 */
export class BehaviorNode extends ExtensionProperty<IBehaviorNode> {
  static override EXTENSION_NAME = EXTENSION_NAME.Behavior;
  declare extensionName: typeof EXTENSION_NAME.Behavior;
  declare propertyType: "BehaviorNode";
  declare parentTypes: [];

  protected init() {
    this.extensionName = EXTENSION_NAME.Behavior;
    this.propertyType = "BehaviorNode";
    this.parentTypes = [];
  }

  protected override getDefaults(): Nullable<IBehaviorNode> {
    return Object.assign(super.getDefaults(), {
      type: null,
      name: null,
      parameters: null,
      flow: null,
    });
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

  set parameters(parameters: BehaviorNodeParameters | null) {
    this.set("parameters", parameters);
  }

  get flow() {
    return this.get("flow");
  }

  set flow(flow: Record<string, BehaviorNode> | null) {
    this.set("flow", flow);
  }
}
