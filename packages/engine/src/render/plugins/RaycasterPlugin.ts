import { Camera, Object3D, Raycaster } from "three";

import { PostMessage } from "../../types";
import { PluginState } from "../RenderWorker";
import { SceneLoader } from "../SceneLoader/SceneLoader";
import { FromRenderMessage, PointerData, ToRenderMessage } from "../types";

export class RaycasterPlugin {
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
    const intersections = this.#raycaster.intersectObject(
      this.#sceneLoader.contents
    );

    // Step through parents until we find a valid object
    let intersected: Object3D | undefined;

    intersections.forEach((intersection) => {
      if (intersected) return;
      intersected = findValidObject(intersection.object, this.#sceneLoader);
    });

    if (intersected) {
      const id = this.#sceneLoader.findId(intersected);

      if (id !== undefined) {
        this.#postMessage({ subject: "clicked_object", data: id });
        return;
      }
    }

    this.#postMessage({ subject: "clicked_object", data: null });
  }
}

/*
 * Step through an object's parents to find a non-internal node
 */
function findValidObject(
  object: Object3D,
  sceneLoader: SceneLoader
): Object3D | undefined {
  const nodeId = sceneLoader.findId(object);

  if (nodeId) {
    const node = sceneLoader.getNode(nodeId);
    if (!node) throw new Error(`Node not found: ${nodeId}`);
    if (!node.isInternal) return object;
  }

  // Try parent
  if (object.parent) return findValidObject(object.parent, sceneLoader);
}
