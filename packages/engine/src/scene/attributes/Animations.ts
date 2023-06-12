import { Animation, AnimationSampler, Document, GLTF } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Scene } from "../Scene";
import { Accessors } from "./Accessors";
import { Attribute } from "./Attribute";
import { Nodes } from "./Nodes";

export interface ChannelJSON {
  targetNodeId: string | null;
  targetPath: GLTF.AnimationChannelTargetPath;
  samplerId: string | null;
}

export interface SamplerJSON {
  id: string;
  inputId: string | null;
  outputId: string | null;
  interpolation: GLTF.AnimationSamplerInterpolation;
}

export interface AnimationJSON {
  channels: ChannelJSON[];
  samplers: SamplerJSON[];
}

/**
 * Stores and manages animations within a {@link Scene}.
 *
 * @group Scene
 */
export class Animations extends Attribute<Animation, AnimationJSON> {
  #doc: Document;
  #node: Nodes;
  #accessor: Accessors;

  store = new Map<string, Animation>();

  constructor(scene: Scene) {
    super();

    this.#doc = scene.doc;
    this.#node = scene.node;
    this.#accessor = scene.accessor;
  }

  getId(animation: Animation) {
    for (const [id, m] of this.store) {
      if (m === animation) return id;
    }
  }

  create(json: Partial<AnimationJSON> = {}, id?: string) {
    const animation = this.#doc.createAnimation();
    this.applyJSON(animation, json);

    const { id: animationId } = this.process(animation, id);

    this.emitCreate(animationId);

    return { id: animationId, object: animation };
  }

  process(animation: Animation, id?: string) {
    const animationId = id ?? nanoid();
    this.store.set(animationId, animation);

    animation.addEventListener("dispose", () => {
      this.store.delete(animationId);
    });

    return { id: animationId };
  }

  processChanges() {
    const changed: Animation[] = [];

    // Add new animations
    this.#doc
      .getRoot()
      .listAnimations()
      .forEach((animation) => {
        const animationId = this.getId(animation);
        if (animationId) return;

        this.process(animation);
        changed.push(animation);
      });

    return changed;
  }

  applyJSON(animation: Animation, json: Partial<AnimationJSON>) {
    const { channels = [], samplers = [] } = json;

    const samplerMap = new Map<string, AnimationSampler>();

    samplers.forEach((samplerJSON) => {
      const { id, inputId, outputId, interpolation } = samplerJSON;

      const input = inputId ? this.#accessor.store.get(inputId) ?? null : null;
      const output = outputId ? this.#accessor.store.get(outputId) ?? null : null;

      const samplerObject = this.#doc.createAnimationSampler();
      samplerObject.setInput(input);
      samplerObject.setOutput(output);
      samplerObject.setInterpolation(interpolation);

      samplerMap.set(id, samplerObject);
      animation.addSampler(samplerObject);
    });

    channels.forEach((channel) => {
      const { targetNodeId, targetPath, samplerId } = channel;

      const target = targetNodeId ? this.#node.store.get(targetNodeId) ?? null : null;
      const sampler = samplerId ? samplerMap.get(samplerId) ?? null : null;

      const channelObject = this.#doc.createAnimationChannel();
      channelObject.setTargetNode(target);
      channelObject.setTargetPath(targetPath);
      channelObject.setSampler(sampler);

      animation.addChannel(channelObject);
    });
  }

  toJSON(animation: Animation): AnimationJSON {
    const channels: ChannelJSON[] = [];
    const samplers: SamplerJSON[] = [];

    const samperIds = new Map<AnimationSampler, string>();

    animation.listSamplers().forEach((sampler) => {
      const input = sampler.getInput();
      const output = sampler.getOutput();

      const inputId = input ? this.#accessor.getId(input) ?? null : null;
      const outputId = output ? this.#accessor.getId(output) ?? null : null;

      const interpolation = sampler.getInterpolation();
      const id = nanoid();
      samperIds.set(sampler, id);

      samplers.push({ id, inputId, interpolation, outputId });
    });

    animation.listChannels().forEach((channel) => {
      const target = channel.getTargetNode();
      const sampler = channel.getSampler();

      const targetNodeId = target ? this.#node.getId(target) ?? null : null;
      const samplerId = sampler ? samperIds.get(sampler) ?? null : null;

      const targetPath = channel.getTargetPath() ?? "translation";

      channels.push({ samplerId, targetNodeId, targetPath });
    });

    return { channels, samplers };
  }
}
