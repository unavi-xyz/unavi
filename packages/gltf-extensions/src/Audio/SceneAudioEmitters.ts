import { ExtensionProperty, IProperty, Nullable, PropertyType } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { AudioEmitter } from "./AudioEmitter";
import { PROPERTY_TYPES } from "./constants";

interface ISceneAudioEmitters extends IProperty {
  emitters: AudioEmitter[];
}

export class SceneAudioEmitters extends ExtensionProperty<ISceneAudioEmitters> {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Audio;
  declare extensionName: typeof EXTENSION_NAME.Audio;
  declare propertyType: typeof PROPERTY_TYPES.SceneAudioEmitters;
  declare parentTypes: [PropertyType.SCENE];

  protected init(): void {
    this.extensionName = EXTENSION_NAME.Audio;
    this.propertyType = PROPERTY_TYPES.SceneAudioEmitters;
    this.parentTypes = [PropertyType.SCENE];
  }

  protected override getDefaults(): Nullable<ISceneAudioEmitters> {
    return Object.assign(super.getDefaults() as IProperty, {
      emitters: [],
    });
  }

  listEmitters(): AudioEmitter[] {
    return this.listRefs("emitters");
  }

  addEmitter(emitter: AudioEmitter): this {
    return this.addRef("emitters", emitter);
  }

  removeEmitter(emitter: AudioEmitter): this {
    return this.removeRef("emitters", emitter);
  }
}
