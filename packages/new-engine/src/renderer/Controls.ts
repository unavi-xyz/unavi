import { Raycaster } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { TransformControls } from "three/examples/jsm/controls/TransformControls";

import { ToRenderMessage } from "../types";
import { RenderWorker } from "./RenderWorker";

export class Controls {
  #orbitControls: OrbitControls | null = null;
  #transformControls: TransformControls | null = null;
  #transformRaycaster: Raycaster | null = null;
  #mouseX = 0;
  #mouseY = 0;
  #usingTransformControls = false;

  #worker: RenderWorker;

  constructor(worker: RenderWorker) {
    this.#worker = worker;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const camera = this.#worker.camera;
    const renderer = this.#worker.renderer;
    const scene = this.#worker.scene;
    if (!camera || !renderer || !scene) return;

    const { subject, data } = event.data;

    switch (subject) {
      case "create_orbit_controls":
        this.#destroyOrbitControls();
        this.#orbitControls = new OrbitControls(camera, renderer.domElement);
        break;
      case "destroy_orbit_controls":
        this.#destroyOrbitControls();
        break;
      case "set_orbit_controls_enabled":
        if (this.#orbitControls) this.#orbitControls.enabled = data.enabled;
        break;
      case "create_transform_controls":
        this.#destroyTransformControls();
        if (camera && renderer) {
          this.#transformControls = new TransformControls(camera, renderer.domElement);
          this.#transformRaycaster = new Raycaster();
          scene.add(this.#transformControls);

          this.#transformControls.addEventListener(
            "mouseDown",
            this.#onTransformMouseDown.bind(this)
          );
          renderer.domElement.addEventListener("mouseup", this.#onCanvasMouseUp.bind(this));
          renderer.domElement.addEventListener("mousedown", this.#onCanvasMouseDown.bind(this));
        }
        break;
      case "destroy_transform_controls":
        this.#destroyTransformControls();
        break;
      case "set_transform_controls_enabled":
        if (this.#transformControls) this.#transformControls.enabled = data.enabled;
        break;
      case "set_transform_controls_mode":
        if (this.#transformControls) this.#transformControls.setMode(data.mode);
        break;
      case "attach_transform_controls":
        if (this.#transformControls) {
          const object = scene.getObjectByProperty("uuid", data.uuid);
          if (object) this.#transformControls.attach(object);
        }
        break;
      case "detach_transform_controls":
        if (this.#transformControls) this.#transformControls.detach();
        break;
    }
  }

  #destroyOrbitControls() {
    if (this.#orbitControls) {
      this.#orbitControls.dispose();
      this.#orbitControls = null;
    }
  }

  #destroyTransformControls() {
    const renderer = this.#worker.renderer;
    const scene = this.#worker.scene;
    if (!renderer || !scene) return;

    if (this.#transformControls) {
      this.#transformControls.removeEventListener("mouseDown", this.#onTransformMouseDown);
      renderer.domElement.removeEventListener("mouseup", this.#onCanvasMouseUp);
      renderer.domElement.removeEventListener("mousedown", this.#onCanvasMouseDown);

      scene.remove(this.#transformControls);
      this.#transformControls.dispose();
      this.#transformControls = null;
      this.#transformRaycaster = null;
    }
  }

  #onTransformMouseDown() {
    this.#usingTransformControls = true;
    if (this.#orbitControls) this.#orbitControls.enabled = false;
  }

  #onCanvasMouseUp() {
    this.#usingTransformControls = false;
    if (this.#orbitControls) this.#orbitControls.enabled = true;
  }

  #onCanvasMouseDown(event: MouseEvent) {
    const renderer = this.#worker.renderer;
    const camera = this.#worker.camera;

    if (!renderer || !camera) return;

    if (!this.#transformRaycaster || !camera || this.#usingTransformControls) return;

    // Get mouse position on the canvas
    const box = renderer.domElement.getBoundingClientRect();
    const x = event.clientX - box.left;
    const y = event.clientY - box.top;

    this.#mouseX = (x / renderer.domElement.scrollWidth) * 2 - 1;
    this.#mouseY = -(y / renderer.domElement.scrollHeight) * 2 + 1;

    // Set raycaster
    this.#transformRaycaster.setFromCamera(
      {
        x: this.#mouseX,
        y: this.#mouseY,
      },
      camera
    );

    // Get intersected objects
    const intersected = this.#transformRaycaster.intersectObject(this.#worker.tree.tree);
    const uuid = intersected.length > 0 ? intersected[0].object.uuid : null;

    // Send to the main thread
    this.#worker.postMessage({
      subject: "click_intersection",
      data: {
        uuid,
      },
    });
  }
}
