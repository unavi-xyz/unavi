import {
  ManualLifecycleEventEmitter,
  registerCoreProfile,
  registerSceneProfile,
  Registry,
} from "@behave-graph/core";

import { Engine } from "../Engine";

export class BehaviorModule {
  #engine: Engine;

  registry = new Registry();
  lifecycle = new ManualLifecycleEventEmitter();

  constructor(engine: Engine) {
    this.#engine = engine;

    registerCoreProfile(this.registry, undefined, this.lifecycle);
    registerSceneProfile(this.registry);
  }
}
