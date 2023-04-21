import { ExtensionProperty, IProperty, Node, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { BehaviorVariable } from "./BehaviorVariable";
import { BehaviorNodeConfiguration, BehaviorNodeParameters } from "./types";

interface IBehaviorNode extends IProperty {
  configuration: BehaviorNodeConfiguration | null;
  flow: { [key: string]: BehaviorNode };
  links: { [key: string]: BehaviorNode };
  nodes: { [key: string]: Node };
  parameters: BehaviorNodeParameters | null;
  type: string;
  variable: BehaviorVariable;
}

/**
 * Represents a single node in the behavior graph.
 *
 * @group KHR_behavior
 * @see {@link BehaviorExtension}
 */
export class BehaviorNode extends ExtensionProperty<IBehaviorNode> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Behavior;
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
      configuration: null,
      flow: {},
      links: {},
      nodes: {},
      parameters: null,
      type: "",
      variable: null,
    });
  }

  getVariable(): BehaviorVariable | null {
    return this.getRef("variable");
  }

  setVariable(variable: BehaviorVariable | null) {
    this.setRef("variable", variable);
  }

  getLink(linkId: string): BehaviorNode | null {
    return this.getRefMap("links", linkId);
  }

  setLink(linkId: string, behaviorNode: BehaviorNode | null) {
    this.setRefMap("links", linkId, behaviorNode);
  }

  getNode(nodeId: string): Node | null {
    return this.getRefMap("nodes", nodeId);
  }

  setNode(nodeId: string, node: Node | null) {
    this.setRefMap("nodes", nodeId, node);
  }

  getConfiguration() {
    return this.get("configuration");
  }

  setConfiguration(configuration: BehaviorNodeConfiguration | null) {
    this.set("configuration", configuration);
  }

  listFlowKeys() {
    return this.listRefMapKeys("flow");
  }

  getFlow(key: string): BehaviorNode | null {
    return this.getRefMap("flow", key);
  }

  setFlow(key: string, behaviorNode: BehaviorNode) {
    this.setRefMap("flow", key, behaviorNode);
  }

  getParameters() {
    return this.get("parameters");
  }

  setParameters(parameters: BehaviorNodeParameters | null) {
    this.set("parameters", parameters);
  }

  getType() {
    return this.get("type");
  }

  setType(type: string) {
    this.set("type", type);
  }
}
