import { Mesh, PerspectiveCamera, Raycaster } from "three";

import { InputMessage, PointerData } from "../../input/messages";
import { PostMessage } from "../../types";
import { FromRenderMessage } from "../messages";
import { RenderScene } from "../RenderScene";

export class RaycastControls {
  #camera: PerspectiveCamera;
  #renderScene: RenderScene;
  #postMessage: PostMessage<FromRenderMessage>;

  #raycaster = new Raycaster();

  #startMoveTime = 0;
  #moveCount = 0;

  constructor(
    camera: PerspectiveCamera,
    renderScene: RenderScene,
    postMessage: PostMessage<FromRenderMessage>
  ) {
    this.#camera = camera;
    this.#renderScene = renderScene;
    this.#postMessage = postMessage;
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

    // if (this.#state.usingTransformControls) return;
    if (!isValidClick || data.button !== 0) return;

    // Move raycaster to pointer position
    this.#raycaster.setFromCamera({ x: data.pointer.x, y: data.pointer.y }, this.#camera);

    // Get intersected objects
    const intersections = this.#raycaster.intersectObject(this.#renderScene.root);

    // Find the first intersected object that is a primitive
    let primitiveId: string | undefined;

    intersections.find((intersection) => {
      if (intersection.object instanceof Mesh) {
        for (const [id, p] of this.#renderScene.primitiveObjects.entries()) {
          if (p === intersection.object) {
            primitiveId = id;
            return true;
          }
        }
      }

      return false;
    });

    if (primitiveId) {
      // Get the mesh that the primitive belongs to
      let meshId: string | undefined;

      for (const [id, m] of this.#renderScene.mesh.store.entries()) {
        if (this.#renderScene.mesh.toJSON(m).primitives.includes(primitiveId)) {
          meshId = id;
        }
      }

      if (meshId) {
        // Get the node that the mesh belongs to
        let nodeId: string | undefined;

        for (const [id, n] of this.#renderScene.node.store.entries()) {
          if (this.#renderScene.node.toJSON(n).mesh === meshId) {
            nodeId = id;
          }
        }

        if (nodeId) {
          this.#postMessage({ subject: "clicked_node", data: { id: nodeId } });
          return;
        }
      }
    }

    this.#postMessage({ subject: "clicked_node", data: { id: null } });
  }
}
