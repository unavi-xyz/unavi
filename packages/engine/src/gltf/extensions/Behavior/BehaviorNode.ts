import { ExtensionProperty, IProperty, Node, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { BehaviorNodeConfiguration, BehaviorNodeParameters } from "./types";
import { Variable } from "./Variable";

interface IBehaviorNode extends IProperty {
  configuration: BehaviorNodeConfiguration | null;
  flow: { [key: string]: BehaviorNode };
  links: { [key: string]: BehaviorNode };
  nodes: { [key: string]: Node };
  parameters: BehaviorNodeParameters | null;
  type: string;
  variable: Variable;
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
      configuration: null,
      flow: {},
      links: {},
      name: null,
      nodes: {},
      parameters: null,
      type: null,
      variable: null,
    });
  }

  get variable() {
    return this.getRef("variable");
  }

  set variable(variable: Variable | null) {
    this.setRef("variable", variable);
  }

  getLink(linkId: string) {
    return this.getRefMap("links", linkId);
  }

  setLink(linkId: string, behaviorNode: BehaviorNode | null) {
    this.setRefMap("links", linkId, behaviorNode);
  }

  getNode(nodeId: string) {
    return this.getRefMap("nodes", nodeId);
  }

  setNode(nodeId: string, node: Node | null) {
    this.setRefMap("nodes", nodeId, node);
  }

  get configuration() {
    return this.get("configuration");
  }

  set configuration(configuration: BehaviorNodeConfiguration | null) {
    this.set("configuration", configuration);
  }

  listFlowKeys() {
    return this.listRefMapKeys("flow");
  }

  getFlow(key: string) {
    return this.getRefMap("flow", key);
  }

  setFlow(key: string, behaviorNode: BehaviorNode) {
    this.setRefMap("flow", key, behaviorNode);
  }

  get parameters() {
    return this.get("parameters");
  }

  set parameters(parameters: BehaviorNodeParameters | null) {
    this.set("parameters", parameters);
  }

  get type() {
    return this.get("type");
  }

  set type(type: string) {
    this.set("type", type);
  }
}
