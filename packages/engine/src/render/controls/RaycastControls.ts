import { PerspectiveCamera, Raycaster } from "three";

import { InputMessage, PointerData } from "../../input/messages";
import { PostMessage } from "../../types";
import { FromRenderMessage } from "../messages";
import { RenderScene } from "../RenderScene";
import { TransformControls } from "./TransformControls";

export class RaycastControls {
  #camera: PerspectiveCamera;
  #renderScene: RenderScene;
  #postMessage: PostMessage<FromRenderMessage>;
  #transformControls: TransformControls;

  #raycaster = new Raycaster();

  #startMoveTime = 0;
  #moveCount = 0;

  constructor(
    camera: PerspectiveCamera,
    renderScene: RenderScene,
    postMessage: PostMessage<FromRenderMessage>,
    transformControls: TransformControls
  ) {
    this.#camera = camera;
    this.#renderScene = renderScene;
    this.#postMessage = postMessage;
    this.#transformControls = transformControls;
  }

  onmessage({ subject, data }: InputMessage) {
    switch (subject) {
      case "pointerdown": {
        this.#startMoveTime = performance.now();
        this.#moveCount = 0;
        break;
      }

      case "pointermove": {
        this.#moveCount += 1;
        break;
      }

      case "pointerup": {
        this.#onPointerUp(data);
        break;
      }
    }
  }

  #onPointerUp(data: PointerData) {
    // Only fire a click if the pointer hasn't moved much, and hasn't been held down for too long
    // This is to prevent clicks from firing when the user is dragging the camera
    const isValidClick = performance.now() - this.#startMoveTime < 1000 && this.#moveCount < 6;

    if (!isValidClick || data.button !== 0 || this.#transformControls.usingControls) return;

    // Move raycaster to pointer position
    this.#raycaster.setFromCamera({ x: data.pointer.x, y: data.pointer.y }, this.#camera);

    // Get intersected objects
    const intersections = this.#raycaster.intersectObject(this.#renderScene.root);

    const intersection = intersections.find((intersection) => {
      // Accept intersections with primitives
      const primitiveId = this.#renderScene.getPrimitiveObjectId(intersection.object);
      if (primitiveId) return true;

      // Accept intersections with custom meshes
      const meshId = this.#renderScene.getCustomMeshObjectId(intersection.object);
      if (meshId) return true;

      return false;
    });

    if (intersection) {
      const nodeId = this.#renderScene.getObjectNodeId(intersection.object);
      if (nodeId) {
        this.#postMessage({ subject: "clicked_node", data: { nodeId } });
        return;
      }
    }

    this.#postMessage({ subject: "clicked_node", data: { nodeId: null } });
  }
}
