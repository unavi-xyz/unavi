import { ExtensionProperty, IProperty, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { AudioData } from "./AudioData";
import { AudioPropertyType } from "./types";

interface IAudioSource extends IProperty {
  gain: number;
  autoPlay: boolean;
  loop: boolean;
  audio: AudioData;
}

/**
 * Represents an audio clip.
 *
 * @group KHR_audio
 * @see {@link AudioExtension}
 */
export class AudioSource extends ExtensionProperty<IAudioSource> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: AudioPropertyType.AudioSource;
  declare parentTypes: [AudioPropertyType.AudioEmitter];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = AudioPropertyType.AudioSource;
    this.parentTypes = [AudioPropertyType.AudioEmitter];
  }

  protected override getDefaults(): Nullable<IAudioSource> {
    return Object.assign(super.getDefaults() as IProperty, {
      autoPlay: false,
      gain: 1,
      loop: false,
      audio: null,
    });
  }

  getGain() {
    return this.get("gain");
  }

  setGain(gain: number) {
    this.set("gain", gain);
    return this;
  }

  getAutoPlay() {
    return this.get("autoPlay");
  }

  setAutoPlay(autoPlay: boolean) {
    this.set("autoPlay", autoPlay);
    return this;
  }

  getLoop() {
    return this.get("loop");
  }

  setLoop(loop: boolean) {
    this.set("loop", loop);
    return this;
  }

  getAudio(): AudioData | null {
    return this.getRef("audio");
  }

  setAudio(audio: AudioData) {
    this.setRef("audio", audio);
    return this;
  }
}
