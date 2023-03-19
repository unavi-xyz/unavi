import { Object3D } from "three";

import { Vec4 } from "../../types";
import { ToRenderMessage } from "../messages";
import { RenderThread } from "../RenderThread";
import { ThreeTransformControls } from "../three/ThreeTransformControls";

export class TransformControls {
  #renderThread: RenderThread;

  #target = new EventTarget();
  #transformControls: ThreeTransformControls;

  usingControls = false;

  constructor(renderThread: RenderThread) {
    this.#renderThread = renderThread;

    this.#transformControls = new ThreeTransformControls(renderThread.camera, this.#target);
    renderThread.scene.add(this.#transformControls);

    this.#transformControls.traverse((object) => {
      object.userData.isTransformControls = true;
    });

    this.#transformControls.addEventListener("mouseDown", () => {
      this.usingControls = true;
    });

    this.#transformControls.addEventListener("mouseUp", () => {
      this.usingControls = false;

      const object = this.#transformControls.object;
      if (!object) return;

      const nodeId = this.#renderThread.renderScene.getNodeId(object);
      if (!nodeId) throw new Error("Node id not found");

      const node = this.#renderThread.renderScene.node.store.get(nodeId);
      if (!node) throw new Error("Node not found");

      // Save transform
      node.setTranslation(object.position.toArray());
      node.setRotation(object.quaternion.toArray() as Vec4);
      node.setScale(object.scale.toArray());

      // Send to main thread
      this.#renderThread.postMessage({
        subject: "set_node_transform",
        data: {
          nodeId,
          translation: object.position.toArray(),
          rotation: object.quaternion.toArray() as Vec4,
          scale: object.scale.toArray(),
        },
      });
    });
  }

  onmessage({ subject, data }: ToRenderMessage) {
    switch (subject) {
      case "set_transform_controls_target": {
        if (data.nodeId === null) {
          // Detach if null
          this.#transformControls.detach();

          // Remove outline pass
          if (this.#renderThread.outlinePass) {
            this.#renderThread.outlinePass.selectedObjects = [];
          }
        } else {
          const object = this.#renderThread.renderScene.builders.node.getObject(data.nodeId);

          if (object) {
            // Attach controls
            if (data.attach) this.#transformControls.attach(object);

            // Add outline pass
            if (this.#renderThread.outlinePass) {
              const selectedObjects: Object3D[] = [];
              selectedObjects.push(object);
              this.#renderThread.outlinePass.selectedObjects = selectedObjects;
            }
          }
        }

        break;
      }

      case "set_transform_controls_mode": {
        this.#transformControls.mode = data;
        break;
      }

      case "pointermove": {
        const pointerMoveEvent = new CustomEvent("pointermove", { detail: data });
        this.#target.dispatchEvent(pointerMoveEvent);
        break;
      }

      case "pointerdown": {
        const pointerDownEvent = new CustomEvent("pointerdown", { detail: data });
        this.#target.dispatchEvent(pointerDownEvent);
        break;
      }

      case "pointerup": {
        const pointerUpEvent = new CustomEvent("pointerup", { detail: data });
        this.#target.dispatchEvent(pointerUpEvent);
        break;
      }
    }
  }

  detach() {
    this.#transformControls.detach();
  }

  destroy() {
    this.detach();
    this.#transformControls.dispose();
  }
}
