import { Camera, Raycaster } from "three";

import { PostMessage } from "../../types";
import { PluginState } from "../RenderWorker";
import { SceneLoader } from "../SceneLoader/SceneLoader";
import { FromRenderMessage, PointerData, ToRenderMessage } from "../types";
import { Plugin } from "./Plugin";

export class RaycasterPlugin extends Plugin {
  #raycaster = new Raycaster();
  #camera: Camera;
  #sceneLoader: SceneLoader;
  #postMessage: PostMessage<FromRenderMessage>;
  #state: PluginState;

  #startMoveTime = 0;
  #moveCount = 0;

  constructor(
    camera: Camera,
    sceneLoader: SceneLoader,
    postMessage: PostMessage<FromRenderMessage>,
    state: PluginState
  ) {
    super();

    this.#camera = camera;
    this.#sceneLoader = sceneLoader;
    this.#postMessage = postMessage;
    this.#state = state;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "pointerdown":
        this.#startMoveTime = performance.now();
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
    // Only fire a click if the pointer hasn't moved much, and hasn't been held down for too long
    // This is to prevent clicks from firing when the user is dragging the camera
    const isValidClick =
      performance.now() - this.#startMoveTime < 1000 && this.#moveCount < 6;

    if (
      data.button !== 0 ||
      this.#state.usingTransformControls ||
      !isValidClick
    )
      return;

    // Move raycaster to pointer position
    this.#raycaster.setFromCamera(
      { x: data.pointer.x, y: data.pointer.y },
      this.#camera
    );

    // Get intersected objects
    const intersected = this.#raycaster.intersectObject(
      this.#sceneLoader.contents
    );

    if (intersected[0]) {
      const id = this.#sceneLoader.findId(intersected[0].object);

      if (id !== undefined) {
        this.#postMessage({ subject: "clicked_object", data: id });
        return;
      }
    }

    this.#postMessage({ subject: "clicked_object", data: null });
  }
}
