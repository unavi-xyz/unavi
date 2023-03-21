import { Intersection, Object3D, Raycaster } from "three";

import { PointerData } from "../../input/messages";
import { ToRenderMessage } from "../messages";
import { RenderThread } from "../RenderThread";

export class RaycastControls {
  #renderThread: RenderThread;

  #raycaster = new Raycaster();

  #startMoveTime = 0;
  #moveCount = 0;
  #hoveredNodeId: string | null = null;

  constructor(renderThread: RenderThread) {
    this.#renderThread = renderThread;
    this.#raycaster.firstHitOnly = true;
  }

  onmessage({ subject, data }: ToRenderMessage) {
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

  update() {
    // Set raycaster to middle of camera position
    this.#raycaster.setFromCamera({ x: 0, y: 0 }, this.#renderThread.camera);

    // Get intersected avatar object
    const avatars = Array.from(this.#renderThread.renderScene.builders.node.avatarObjects.values());
    const intersections = this.#raycaster.intersectObjects(avatars);

    let nodeId: string | null = null;

    intersections.find((intersection) => {
      const avatarNodeId = this.#renderThread.renderScene.getAvatarNodeId(intersection.object);
      if (avatarNodeId) {
        nodeId = avatarNodeId;
        return true;
      }

      return false;
    });

    if (nodeId !== this.#hoveredNodeId) {
      this.#hoveredNodeId = nodeId;

      this.#renderThread.postMessage({ subject: "hovered_node", data: { nodeId, isAvatar: true } });

      if (this.#renderThread.outlinePass) {
        this.#renderThread.outlinePass.selectedObjects = [];

        if (nodeId) {
          const avatarObject =
            this.#renderThread.renderScene.builders.node.avatarObjects.get(nodeId);

          if (avatarObject) {
            // Highlight object
            this.#renderThread.outlinePass.selectedObjects = [avatarObject];
          }
        }
      }
    }
  }

  #onPointerUp(data: PointerData) {
    if (this.#renderThread.controls === "orbit") {
      // Only fire a click if the pointer hasn't moved much, and hasn't been held down for too long
      // This is to prevent clicks from firing when the user is dragging the camera
      const isValidClick = performance.now() - this.#startMoveTime < 1000 && this.#moveCount < 6;
      if (!isValidClick || data.button !== 0 || this.#renderThread.transform.usingControls) return;
    }

    // Move raycaster to camera position
    this.#raycaster.setFromCamera(
      this.#renderThread.controls === "orbit"
        ? { x: data.pointer.x, y: data.pointer.y }
        : { x: 0, y: 0 },
      this.#renderThread.camera
    );

    // Get intersected object
    const intersections = this.#raycaster.intersectObject(this.#renderThread.renderScene.root);
    const { nodeId, isAvatar } = this.#findIntersection(intersections);

    this.#renderThread.postMessage({ subject: "clicked_node", data: { nodeId, isAvatar } });
  }

  #findIntersection(intersections: Intersection<Object3D>[]) {
    let nodeId: string | null = null;
    let isAvatar = false;

    const intersection = intersections.find((intersection) => {
      const avatarNodeId = this.#renderThread.renderScene.getAvatarNodeId(intersection.object);
      if (avatarNodeId) {
        nodeId = avatarNodeId;
        isAvatar = true;
        return true;
      }

      nodeId = this.#renderThread.renderScene.getObjectNodeId(intersection.object);
      if (nodeId) return true;

      return false;
    });

    return { intersection, nodeId, isAvatar };
  }
}
