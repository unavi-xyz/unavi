import { EventEmitter } from "@unavi/behave-graph-core";

import { RenderScene } from "../RenderScene";

type Event<T extends string, D> = { type: T; data: D };

export type BuilderEvent = Event<"objectChange", string>;

type EventType = BuilderEvent["type"];
type EventData = BuilderEvent["data"];

/**
 * @internal
 * Base class for converting gltf data to Three.js objects.
 */
export abstract class Builder<T, O> extends EventEmitter<BuilderEvent> {
  protected scene: RenderScene;

  #objects = new Map<string, O>();

  constructor(scene: RenderScene) {
    super();

    this.scene = scene;
  }

  getObject(id: string) {
    return this.#objects.get(id);
  }

  listObjects() {
    return Array.from(this.#objects.entries());
  }

  setObject(id: string, object: O | null) {
    if (object) this.#objects.set(id, object);
    else this.#objects.delete(id);

    this.emit({ type: "objectChange", data: id });
  }

  /**
   * Subscribe to an event.
   * @param type The type of event to subscribe to.
   * @param fn The function to call when the event is emitted.
   * @returns A function to unsubscribe.
   */
  subscribe(type: EventType, fn: (data: EventData) => void) {
    const listener = (e: BuilderEvent) => {
      if (e.type === type) {
        fn(e.data);
      }
    };

    this.addListener(listener);

    return () => {
      this.removeListener(listener);
    };
  }

  /**
   * Subscribe to changes to a specific object.
   * @param id The id of the object to subscribe to.
   * @param fn The function to call when the object changes. The function may return a cleanup function.
   * @returns A function to unsubscribe.
   */
  subscribeToObject(id: string, fn: (data: O | undefined) => (() => void) | void) {
    let cleanup: (() => void) | void;

    const onChange = (changedId: string) => {
      if (changedId === id) {
        if (cleanup) {
          cleanup();
          cleanup = undefined;
        }

        cleanup = fn(this.#objects.get(id));
      }
    };

    // Set initial value
    onChange(id);

    return this.subscribe("objectChange", onChange);
  }

  abstract add(json: Partial<T>, id: string): void;

  abstract remove(id: string): void;

  abstract update(json: Partial<T>, id: string): void;
}
