import { ExtensionProperty, IProperty, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { PositionalEmitterDef } from "./schemas";
import { Source } from "./Source";

interface IEmitter extends IProperty {
  type: "positional" | "global" | string;
  gain: number;
  sources: { [sourceId: string]: Source };
  positional: PositionalEmitterDef;
}

/**
 * Represents an audio emitter within the scene.
 *
 * @group KHR_audio
 * @see {@link AudioExtension}
 */
export class Emitter extends ExtensionProperty<IEmitter> {
  static override EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: "Emitter";
  declare parentTypes: [];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = "Emitter";
    this.parentTypes = [];
  }

  protected override getDefaults(): Nullable<IEmitter> {
    return Object.assign(super.getDefaults(), {
      type: null,
      gain: null,
      sources: {},
      positional: {},
    });
  }

  getType() {
    return this.get("type");
  }

  setType(type: string) {
    this.set("type", type);
    return this;
  }

  getGain() {
    return this.get("gain");
  }

  setGain(gain: number) {
    this.set("gain", gain);
    return this;
  }

  getSource(sourceId: string) {
    return this.getRefMap("sources", sourceId);
  }

  setSource(sourceId: string, node: Source | null) {
    this.setRefMap("sources", sourceId, node);
  }

  listSources() {
    return this.listRefMapValues("sources");
  }

  getPositional() {
    return this.get("positional");
  }

  setPositional(positional: PositionalEmitterDef) {
    this.set("positional", positional);
    return this;
  }
}
