import { Object3D, ObjectLoader } from "three";

import { FakeWorker } from "./classes/FakeWorker";
import { RenderWorker } from "./renderer/RenderWorker";
import { FromRenderMessage, ToRenderMessage } from "./types";

export class RenderThread {
  #worker: Worker | FakeWorker<ToRenderMessage, FromRenderMessage>;
  #messageId = 1;
  #listeners = new Map<number, (message: FromRenderMessage) => void>();

  constructor(canvas: HTMLCanvasElement) {
    // Render.worker.ts is broken atm
    // I think webpack / next.js is messing with the worker
    // It is setting "window" in the worker to be typeof object, instead of typeof undefined
    // so three.js tries to access it and throws an error :/

    // If OffscreenCanvas is supported, render in a worker
    // const offscreen = canvas.transferControlToOffscreen();
    // this.#worker = new Worker(new URL("./workers/Render.worker.ts", import.meta.url), {
    //   type: "module",
    // });
    // this.#worker.postMessage({ type: "init", data: { canvas: offscreen } }, [offscreen]);
    // Otherwise, render in the main thread

    const renderWorker = new RenderWorker(canvas);
    this.#worker = new FakeWorker(renderWorker.onmessage.bind(renderWorker));
    renderWorker.postMessage = this.#worker.workerPort.postMessage.bind(this.#worker.workerPort);

    this.#worker.onmessage = (event: MessageEvent<FromRenderMessage>) => {
      const { id, subject, data } = event.data;

      // Send event to listener
      const listener = id ? this.#listeners.get(id) : null;
      if (id && listener) {
        listener(event.data);
        this.#listeners.delete(id);
      }

      switch (subject) {
        case "click_intersection":
          this.onClickIntersection(data.uuid);
          break;
      }
    };
  }

  onClickIntersection(uuid: string | null) {}

  setObject(object: Object3D) {
    const json = object.toJSON();
    this.#worker.postMessage({ subject: "set_object", data: { json } });
  }

  addObject(object: Object3D, parent?: string | null) {
    const json = object.toJSON();
    if (parent) {
      this.#worker.postMessage({ subject: "add_object", data: { json, parent } });
    } else {
      this.#worker.postMessage({ subject: "add_object", data: { json } });
    }
  }

  removeObject(uuid: string) {
    this.#worker.postMessage({ subject: "remove_object", data: { uuid } });
  }

  moveObject(uuid: string, parent: string) {
    this.#worker.postMessage({ subject: "move_object", data: { uuid, parent } });
  }

  getObject(uuid: string) {
    const id = this.#messageId++;

    const promise = new Promise<Object3D>((resolve, reject) => {
      this.#listeners.set(id, (message: FromRenderMessage) => {
        if (message.subject === "got_object") {
          const loader = new ObjectLoader();

          if (message.data.json) {
            const object = loader.parse(message.data.json);
            resolve(object);
          }

          reject(new Error("No object found"));
        }
      });
    });

    this.#worker.postMessage({ id, subject: "get_object", data: { uuid } });

    return promise;
  }

  destroy() {
    this.#worker.postMessage({ subject: "destroy", data: null });
  }

  // Orbit controls
  createOrbitControls() {
    this.#worker.postMessage({ subject: "create_orbit_controls", data: null });
  }

  destroyOrbitControls() {
    this.#worker.postMessage({ subject: "destroy_orbit_controls", data: null });
  }

  setOrbitControlsEnabled(enabled: boolean) {
    this.#worker.postMessage({ subject: "set_orbit_controls_enabled", data: { enabled } });
  }

  // Transform controls
  createTransformControls() {
    this.#worker.postMessage({ subject: "create_transform_controls", data: null });
  }

  destroyTransformControls() {
    this.#worker.postMessage({ subject: "destroy_transform_controls", data: null });
  }

  setTransformControlsEnabled(enabled: boolean) {
    this.#worker.postMessage({ subject: "set_transform_controls_enabled", data: { enabled } });
  }

  setTransformControlsMode(mode: "translate" | "rotate" | "scale") {
    this.#worker.postMessage({ subject: "set_transform_controls_mode", data: { mode } });
  }

  attachTransformControls(uuid: string) {
    this.#worker.postMessage({ subject: "attach_transform_controls", data: { uuid } });
  }

  detachTransformControls() {
    this.#worker.postMessage({ subject: "detach_transform_controls", data: null });
  }

  // Skybox
  createSkybox(path: string) {
    this.#worker.postMessage({ subject: "create_skybox", data: { path } });
  }
}
