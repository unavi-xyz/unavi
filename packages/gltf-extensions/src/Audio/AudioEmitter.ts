import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { AudioSource } from "./AudioSource";
import { AudioEmitterDistanceModel, AudioEmitterType } from "./schemas";
import { AudioPropertyType } from "./types";

interface IAudioEmitter extends IProperty {
  type: AudioEmitterType;
  gain: number;
  sources: AudioSource[];
  coneInnerAngle: number;
  coneOuterAngle: number;
  coneOuterGain: number;
  distanceModel: AudioEmitterDistanceModel;
  maxDistance: number;
  refDistance: number;
  rolloffFactor: number;
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

  static Type: Record<string, AudioEmitterType> = {
    GLOBAL: "global",
    POSITIONAL: "positional",
  };

  static DistanceModel: Record<string, AudioEmitterDistanceModel> = {
    INVERSE: "inverse",
    LINEAR: "linear",
    EXPONENTIAL: "exponential",
  };

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = AudioPropertyType.AudioEmitter;
    this.parentTypes = [PropertyType.NODE, AudioPropertyType.SceneAudioEmitters];
  }

  protected override getDefaults(): Nullable<IAudioEmitter> {
    return Object.assign(super.getDefaults() as IProperty, {
      type: "global" as const,
      gain: 1,
      sources: [],
      coneInnerAngle: Math.PI * 2,
      coneOuterAngle: Math.PI * 2,
      coneOuterGain: 0,
      distanceModel: "inverse" as const,
      maxDistance: 10000,
      refDistance: 1,
      rolloffFactor: 1,
    });
  }

  getType() {
    return this.get("type");
  }

  setType(type: AudioEmitterType) {
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

  getConeInnerAngle(): number {
    return this.get("coneInnerAngle");
  }

  setConeInnerAngle(coneInnerAngle: number): this {
    return this.set("coneInnerAngle", coneInnerAngle);
  }

  getConeOuterAngle(): number {
    return this.get("coneOuterAngle");
  }

  setConeOuterAngle(coneOuterAngle: number): this {
    return this.set("coneOuterAngle", coneOuterAngle);
  }

  getConeOuterGain(): number {
    return this.get("coneOuterGain");
  }

  setConeOuterGain(coneOuterGain: number): this {
    return this.set("coneOuterGain", coneOuterGain);
  }

  getDistanceModel(): AudioEmitterDistanceModel {
    return this.get("distanceModel");
  }

  setDistanceModel(distanceModel: AudioEmitterDistanceModel): this {
    return this.set("distanceModel", distanceModel);
  }

  getMaxDistance(): number {
    return this.get("maxDistance");
  }

  setMaxDistance(maxDistance: number): this {
    return this.set("maxDistance", maxDistance);
  }

  getRefDistance(): number {
    return this.get("refDistance");
  }

  setRefDistance(refDistance: number): this {
    return this.set("refDistance", refDistance);
  }

  getRolloffFactor(): number {
    return this.get("rolloffFactor");
  }

  setRolloffFactor(rolloffFactor: number): this {
    return this.set("rolloffFactor", rolloffFactor);
  }
}
