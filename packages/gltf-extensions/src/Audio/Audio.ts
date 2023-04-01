import { ExtensionProperty, IProperty, Nullable } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";

interface IAudio extends IProperty {
  uri: string;
}

/**
 * Represents audio data (such as an audio file).
 *
 * @group KHR_audio
 * @see {@link AudioExtension}
 */
export class Audio extends ExtensionProperty<IAudio> {
  static override EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: "Audio";
  declare parentTypes: [];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = "Audio";
    this.parentTypes = [];
  }

  protected override getDefaults(): Nullable<IAudio> {
    return Object.assign(super.getDefaults(), {
      uri: null,
    });
  }

  getURI() {
    return this.get("uri");
  }

  setURI(uri: string) {
    this.set("uri", uri);
    return this;
  }
}
