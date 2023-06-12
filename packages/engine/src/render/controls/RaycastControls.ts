import { Intersection, Object3D, Raycaster, Vector2 } from "three";

import { MAX_AVATAR_EQUIP_DISTANCE } from "../../constants";
import { PointerData } from "../../input/messages";
import { ToRenderMessage } from "../messages";
import { RenderThread } from "../RenderThread";

export class RaycastControls {
  #renderThread: RenderThread;

  #raycaster = new Raycaster();
  #vec2 = new Vector2();

  #startMoveTime = 0;
  #moveCount = 0;
  #hoveredNodeId: string | null = null;
  #hoveredWithinDistance = false;

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
    this.#vec2.set(0, 0);
    this.#raycaster.setFromCamera(this.#vec2, this.#renderThread.camera);

    const intersections = this.#raycaster.intersectObject(this.#renderThread.renderScene.root);
    const { intersection, isAvatar, nodeId } = this.#findIntersection(intersections);

    const withinDistance = intersection
      ? intersection.distance <= MAX_AVATAR_EQUIP_DISTANCE
      : false;

    if (nodeId !== this.#hoveredNodeId || withinDistance !== this.#hoveredWithinDistance) {
      this.#hoveredNodeId = nodeId;
      this.#hoveredWithinDistance = withinDistance;

      this.#renderThread.postMessage({
        data: { distance: intersection?.distance ?? null, isAvatar, nodeId },
        subject: "hovered_node",
      });

      if (this.#renderThread.outlinePass) {
        this.#renderThread.outlinePass.selectedObjects = [];

        if (nodeId && withinDistance) {
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
    if (this.#renderThread.controls === "orbit") {
      this.#vec2.set(data.pointer.x, data.pointer.y);
    } else {
      this.#vec2.set(0, 0);
    }

    this.#raycaster.setFromCamera(this.#vec2, this.#renderThread.camera);

    // Get intersected object
    const intersections = this.#raycaster.intersectObject(this.#renderThread.renderScene.root);
    const { intersection, nodeId, isAvatar } = this.#findIntersection(intersections);

    this.#renderThread.postMessage({
      data: { distance: intersection?.distance ?? null, isAvatar, nodeId },
      subject: "clicked_node",
    });
  }

  /**
   * Finds the first valid intersection in the list of intersections.
   * A valid intersection is one that can be traced to a node.
   */
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

    return { intersection, isAvatar, nodeId };
  }
}
