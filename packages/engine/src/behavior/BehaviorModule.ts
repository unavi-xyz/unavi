import {
  DefaultLogger,
  Engine as BehaviorEngine,
  ManualLifecycleEventEmitter,
  readGraphFromJSON,
  registerCoreProfile,
  registerLifecycleEventEmitter,
  registerLogger,
  registerSceneDependency,
  registerSceneProfile,
  Registry,
  validateGraph,
} from "@wired-labs/behave-graph-core";

import { Engine } from "../Engine";
import { BehaviorScene } from "./BehaviorScene";

const TICK_HZ = 60;

/**
 * Handles the lifecycle of the behavior engine.
 *
 * @group Modules
 */
export class BehaviorModule {
  #engine: Engine;

  #behaviorEngine: BehaviorEngine | null = null;
  #interval: NodeJS.Timeout | null = null;

  registry = new Registry();
  lifecycle = new ManualLifecycleEventEmitter();

  constructor(engine: Engine) {
    this.#engine = engine;

    registerCoreProfile(this.registry);
    registerSceneProfile(this.registry);
    registerSceneDependency(this.registry.dependencies, new BehaviorScene(engine));
    registerLogger(this.registry.dependencies, new DefaultLogger());
    registerLifecycleEventEmitter(this.registry.dependencies, this.lifecycle);
  }

  #loadEngine() {
    const json = this.#engine.scene.extensions.behavior.toGraphJSON();

    const graph = readGraphFromJSON(json, this.registry);
    const errors = validateGraph(graph);

    if (errors.length > 0) {
      errors.forEach((error) => console.error(error));
      throw new Error("Invalid graph");
    }

    if (this.#behaviorEngine) this.#behaviorEngine.dispose();
    this.#behaviorEngine = new BehaviorEngine(graph);
  }

  /**
   * Loads the behavior graph from the scene, and starts the engine.
   * If the engine is already running, it will be stopped and restarted.
   */
  start() {
    this.stop();

    this.#loadEngine();

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
