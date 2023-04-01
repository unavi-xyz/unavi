import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { Audio } from "./Audio";
import { Emitter } from "./Emitter";
import { AudioExtensionDef, audioExtensionSchema, EmitterDef } from "./schemas";
import { Source } from "./Source";

/**
 * Implementation of the {@link https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/KHR_audio KHR_audio} extension.
 *
 * @group KHR_audio
 */
export class AudioExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.Audio;
  override readonly extensionName = EXTENSION_NAME.Audio;

  createAudio(): Audio {
    return new Audio(this.document.getGraph());
  }

  createSource(): Source {
    return new Source(this.document.getGraph());
  }

  createEmitter(): Emitter {
    return new Emitter(this.document.getGraph());
  }

  read(context: ReaderContext) {
    if (!context.jsonDoc.json.extensions || !context.jsonDoc.json.extensions[this.extensionName])
      return this;

    const rootDef = audioExtensionSchema.parse(context.jsonDoc.json.extensions[this.extensionName]);

    rootDef.audio.forEach((audioDef) => {
      const audio = this.createAudio();

      // Only support URI for now
      // I'm not sure how to handle buffer views with gltf-transform
      if (audioDef.uri) audio.setURI(audioDef.uri);
      else console.warn("Audio buffer not currently supported");
    });

    rootDef.sources.forEach((sourceDef) => {
      const source = this.createSource();
      source.setAutoPlay(sourceDef.autoPlay);
      source.setGain(sourceDef.gain);
      source.setLoop(sourceDef.loop);
      source.setName(sourceDef.name);

      const audio = this.listAudios()[sourceDef.audio];
      if (!audio) return;

      source.setAudio(audio);
    });

    rootDef.emitters.forEach((emitterDef) => {
      const emitter = this.createEmitter();
      emitter.setType(emitterDef.type);
      emitter.setGain(emitterDef.gain);

      emitterDef.sources.map((sourceDef, i) => {
        const source = this.listSources()[sourceDef];
        if (!source) return;

        emitter.setSource(String(i), source);
      });

      if (emitterDef.positional) emitter.setPositional(emitterDef.positional);
    });

    return this;
  }

  write(context: WriterContext) {
    const rootDef: AudioExtensionDef = {
      audio: [],
      sources: [],
      emitters: [],
    };

    this.listAudios().forEach((audio) => {
      const audioDef = {
        uri: audio.getURI(),
      };

      rootDef.audio.push(audioDef);
    });

    this.listSources().forEach((source) => {
      const audio = source.getAudio();
      if (!audio) return;

      const sourceDef = {
        autoPlay: source.getAutoPlay(),
        gain: source.getGain(),
        loop: source.getLoop(),
        name: source.getName(),
        audio: this.listAudios().indexOf(audio),
      };

      rootDef.sources.push(sourceDef);
    });

    this.listEmitters().forEach((emitter) => {
      const emitterDef: EmitterDef = {
        name: emitter.getName(),
        type: emitter.getType(),
        gain: emitter.getGain(),
        sources: [],
      };

      emitter.listSources().forEach((source) => {
        emitterDef.sources.push(this.listSources().indexOf(source as Source));
      });

      if (emitter.getPositional()) emitterDef.positional = emitter.getPositional();

      rootDef.emitters.push(emitterDef);
    });

    if (rootDef.audio.length || rootDef.sources.length || rootDef.emitters.length) {
      context.jsonDoc.json.extensions = context.jsonDoc.json.extensions || {};
      context.jsonDoc.json.extensions[this.extensionName] = rootDef;
    }

    return this;
  }

  listAudios(): Audio[] {
    return this.listProperties().filter((property): property is Audio => property instanceof Audio);
  }

  listSources(): Source[] {
    return this.listProperties().filter(
      (property): property is Source => property instanceof Source
    );
  }

  listEmitters(): Emitter[] {
    return this.listProperties().filter(
      (property): property is Emitter => property instanceof Emitter
    );
  }
}
