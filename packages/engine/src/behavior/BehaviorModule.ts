import {
  Engine as BehaviorEngine,
  GraphJSON,
  ManualLifecycleEventEmitter,
  readGraphFromJSON,
  registerCoreProfile,
  registerSceneProfile,
  Registry,
  validateGraph,
} from "@behave-graph/core";

import { Engine } from "../Engine";
import { BehaviorScene } from "./BehaviorScene";

const TICK_HZ = 30;

export class BehaviorModule {
  #engine: Engine;

  #behaviorEngine: BehaviorEngine | null = null;
  #interval: NodeJS.Timeout | null = null;

  registry = new Registry();
  lifecycle = new ManualLifecycleEventEmitter();

  constructor(engine: Engine) {
    this.#engine = engine;

    registerCoreProfile(this.registry, undefined, this.lifecycle);
    registerSceneProfile(this.registry, new BehaviorScene(engine));
  }

  get behaviorEngine() {
    return this.#behaviorEngine;
  }

  set behaviorEngine(behaviorEngine: BehaviorEngine | null) {
    if (this.#behaviorEngine) this.#behaviorEngine.dispose();
    this.#behaviorEngine = behaviorEngine;
  }

  readJSON(json: GraphJSON) {
    const graph = readGraphFromJSON(json, this.registry);
    const errors = validateGraph(graph);

    if (errors.length > 0) {
      errors.forEach((error) => console.error(error));
      throw new Error("Invalid graph");
    }

    this.behaviorEngine = new BehaviorEngine(graph);
  }

  start() {
    this.stop();
    this.lifecycle.startEvent.emit();
    this.#execute();
    this.#interval = setInterval(() => this.#tick(), 1000 / TICK_HZ);
  }

  stop() {
    if (this.#interval) clearInterval(this.#interval);
    this.lifecycle.endEvent.emit();
    this.#execute();
  }

  #tick() {
    this.lifecycle.tickEvent.emit();
    this.#execute();
  }

  #execute() {
    this.#behaviorEngine?.executeAllSync(1 / TICK_HZ);
  }
}
