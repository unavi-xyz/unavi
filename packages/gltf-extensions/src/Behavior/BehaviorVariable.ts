import { ExtensionProperty, IProperty, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { ValueType } from "./types";

interface IBehaviorVariable extends IProperty {
  type: string;
  initialValue: any;
}

/**
 * Represents a variable in the behavior graph.
 *
 * @group KHR_behavior
 * @see {@link BehaviorExtension}
 */
export class BehaviorVariable extends ExtensionProperty<IBehaviorVariable> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Behavior;
  declare extensionName: typeof EXTENSION_NAME.Behavior;
  declare propertyType: "Variable";
  declare parentTypes: [];

  protected init() {
    this.extensionName = EXTENSION_NAME.Behavior;
    this.propertyType = "Variable";
    this.parentTypes = [];
  }

  protected override getDefaults(): Nullable<IBehaviorVariable> {
    return Object.assign(super.getDefaults(), {
      type: ValueType.string,
      initialValue: "",
    });
  }

  get name() {
    return this.getName();
  }

  get type() {
    return this.get("type");
  }

  set type(type: string) {
    this.set("type", type);

    switch (type) {
      case ValueType.string: {
        this.initialValue = "";
        break;
      }

      case ValueType.number:
      case ValueType.integer:
      case ValueType.float: {
        this.initialValue = 0;
        break;
      }

      case ValueType.boolean: {
        this.initialValue = false;
        break;
      }

      case ValueType.vec2: {
        this.initialValue = { x: 0, y: 0 };
        break;
      }

      case ValueType.vec3: {
        this.initialValue = { x: 0, y: 0, z: 0 };
        break;
      }

      case ValueType.quat:
      case ValueType.vec4: {
        this.initialValue = { x: 0, y: 0, z: 0, w: 0 };
        break;
      }
    }
  }

  get initialValue() {
    return this.get("initialValue");
  }

  set initialValue(initialValue: any) {
    this.set("initialValue", initialValue);
  }
}
