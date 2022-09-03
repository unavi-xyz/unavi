import { Camera, Raycaster } from "three";

import { PostMessage } from "../../types";
import { PluginState } from "../RenderWorker";
import { SceneManager } from "../classes/SceneManager";
import { FromRenderMessage, PointerData, ToRenderMessage } from "../types";
import { Plugin } from "./Plugin";

export class RaycasterPlugin extends Plugin {
  #raycaster = new Raycaster();
  #camera: Camera;
  #sceneManager: SceneManager;
  #postMessage: PostMessage<FromRenderMessage>;
  #state: PluginState;

  #moveCount = 0;

  constructor(
    camera: Camera,
    sceneManager: SceneManager,
    postMessage: PostMessage<FromRenderMessage>,
    state: PluginState
  ) {
    super();

    this.#camera = camera;
    this.#sceneManager = sceneManager;
    this.#postMessage = postMessage;
    this.#state = state;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "pointerdown":
        this.#moveCount = 0;
        break;
      case "pointermove":
        this.#moveCount += 1;
        break;
      case "pointerup":
        this.#onPointerUp(data);
        break;
    }
  }

  #onPointerUp(data: PointerData) {
    if (
      data.button !== 0 ||
      this.#state.usingTransformControls ||
      this.#moveCount > 20
    )
      return;

    // Move raycaster to pointer position
    this.#raycaster.setFromCamera(
      { x: data.pointer.x, y: data.pointer.y },
      this.#camera
    );

    // Get intersected objects
    const intersected = this.#raycaster.intersectObject(
      this.#sceneManager.root
    );

    if (intersected.length > 0) {
      const object = intersected[0].object;
      const id = this.#sceneManager.findId(object);

      if (id !== undefined) {
        this.#postMessage({ subject: "clicked_object", data: id });
        return;
      }
    }

    this.#postMessage({ subject: "clicked_object", data: null });
  }
}
