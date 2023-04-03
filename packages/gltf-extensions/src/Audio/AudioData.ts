import {
  BufferUtils,
  ExtensionProperty,
  FileUtils,
  IProperty,
  Nullable,
} from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { PROPERTY_TYPES } from "./constants";

interface IAudio extends IProperty {
  uri: string;
  mimeType: string;
  data: Uint8Array | null;
}

/**
 * Represents audio data (such as an audio file).
 *
 * @group KHR_audio
 * @see {@link AudioExtension}
 */
export class AudioData extends ExtensionProperty<IAudio> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: typeof PROPERTY_TYPES.AudioData;
  declare parentTypes: [typeof PROPERTY_TYPES.AudioSource];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = PROPERTY_TYPES.AudioData;
    this.parentTypes = [PROPERTY_TYPES.AudioSource];
  }

  protected override getDefaults(): Nullable<IAudio> {
    return Object.assign(super.getDefaults(), {
      uri: "",
      mimeType: "",
      data: null,
    });
  }

  getURI() {
    return this.get("uri");
  }

  setURI(uri: string) {
    this.set("uri", uri);
    return this;
  }

  public getMimeType(): string {
    return this.get("mimeType") || extensionToMimeType(FileUtils.extension(this.getURI()));
  }

  public setMimeType(mimeType: string): this {
    return this.set("mimeType", mimeType);
  }

  public getData(): Uint8Array | null {
    return this.get("data");
  }

  public setData(data: Uint8Array | null): this {
    return this.set("data", BufferUtils.assertView(data));
  }
}

function extensionToMimeType(extension: string) {
  if (extension === "mp3") return "audio/mpeg";
  return `audio/${extension}`;
}
