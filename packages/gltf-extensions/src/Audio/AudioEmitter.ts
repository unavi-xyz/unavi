import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { AudioSource } from "./AudioSource";
import { PositionalEmitterDef } from "./schemas";
import { AudioPropertyType } from "./types";

interface IAudioEmitter extends IProperty {
  type: "positional" | "global" | string;
  gain: number;
  sources: AudioSource[];
  positional: PositionalEmitterDef;
}

/**
 * Represents an audio emitter within the scene.
 *
 * @group KHR_audio
 * @see {@link AudioExtension}
 */
export class AudioEmitter extends ExtensionProperty<IAudioEmitter> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: typeof AudioPropertyType.AudioEmitter;
  declare parentTypes: [PropertyType.NODE, AudioPropertyType.SceneAudioEmitters];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = AudioPropertyType.AudioEmitter;
    this.parentTypes = [PropertyType.NODE, AudioPropertyType.SceneAudioEmitters];
  }

  protected override getDefaults(): Nullable<IAudioEmitter> {
    return Object.assign(super.getDefaults(), {});
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

  listSources(): AudioSource[] {
    return this.listRefs("sources");
  }

  addSource(source: AudioSource) {
    return this.addRef("sources", source);
  }

  removeSource(source: AudioSource) {
    return this.removeRef("sources", source);
  }

  getPositional() {
    return this.get("positional");
  }

  setPositional(positional: PositionalEmitterDef) {
    this.set("positional", positional);
    return this;
  }
}
