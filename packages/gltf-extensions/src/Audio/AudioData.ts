import {
  BufferUtils,
  ExtensionProperty,
  FileUtils,
  IProperty,
  Nullable,
} from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { AudioPropertyType } from "./types";

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
  declare propertyType: AudioPropertyType.AudioData;
  declare parentTypes: [AudioPropertyType.AudioSource];

  protected init() {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = AudioPropertyType.AudioData;
    this.parentTypes = [AudioPropertyType.AudioSource];
  }

  protected override getDefaults(): Nullable<IAudio> {
    return Object.assign(super.getDefaults() as IProperty, {
      data: null,
      mimeType: "",
      uri: "",
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
    return (
      this.get("mimeType") ||
      extensionToMimeType(FileUtils.extension(this.getURI()))
    );
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
