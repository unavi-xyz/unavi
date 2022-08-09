import { Camera, Object3D, Raycaster, Renderer, Scene } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { TransformControls } from "three/examples/jsm/controls/TransformControls";

import { ToRenderMessage } from "../types";
import { FromRenderPostMessage } from "./RenderWorker";

export class Controls {
  #orbitControls: OrbitControls | null = null;
  #transformControls: TransformControls | null = null;
  #transformRaycaster: Raycaster | null = null;
  #mouseX = 0;
  #mouseY = 0;
  #usingTransformControls = false;

  #camera: Camera;
  #renderer: Renderer;
  #canvas: HTMLCanvasElement;
  #scene: Scene;
  #objects: Object3D;
  #postMessage: FromRenderPostMessage;

  constructor(
    camera: Camera,
    renderer: Renderer,
    scene: Scene,
    objects: Object3D,
    postMessage: FromRenderPostMessage
  ) {
    this.#camera = camera;
    this.#renderer = renderer;
    this.#canvas = renderer.domElement;
    this.#scene = scene;
    this.#objects = objects;
    this.#postMessage = postMessage;
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { subject, data } = event.data;

    switch (subject) {
      case "create_orbit_controls":
        this.#destroyOrbitControls();
        this.#orbitControls = new OrbitControls(this.#camera, this.#renderer.domElement);
        break;
      case "destroy_orbit_controls":
        this.#destroyOrbitControls();
        break;
      case "set_orbit_controls_enabled":
        if (this.#orbitControls) this.#orbitControls.enabled = data.enabled;
        break;
      case "create_transform_controls":
        this.#destroyTransformControls();
        if (this.#camera && this.#renderer) {
          this.#transformControls = new TransformControls(this.#camera, this.#canvas);
          this.#transformRaycaster = new Raycaster();
          this.#scene.add(this.#transformControls);

          this.#transformControls.addEventListener(
            "mouseDown",
            this.#onTransformMouseDown.bind(this)
          );
          this.#canvas.addEventListener("mouseup", this.#onCanvasMouseUp.bind(this));
          this.#canvas.addEventListener("mousedown", this.#onCanvasMouseDown.bind(this));
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
          const object = this.#scene.getObjectByProperty("uuid", data.uuid);
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
    if (this.#transformControls) {
      this.#transformControls.removeEventListener("mouseDown", this.#onTransformMouseDown);
      this.#renderer.domElement.removeEventListener("mouseup", this.#onCanvasMouseUp);
      this.#renderer.domElement.removeEventListener("mousedown", this.#onCanvasMouseDown);

      this.#scene.remove(this.#transformControls);
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
    if (!this.#transformRaycaster || !this.#camera || this.#usingTransformControls) return;

    // Get mouse position on the canvas
    const box = this.#canvas.getBoundingClientRect();
    const x = event.clientX - box.left;
    const y = event.clientY - box.top;

    this.#mouseX = (x / this.#canvas.scrollWidth) * 2 - 1;
    this.#mouseY = -(y / this.#canvas.scrollHeight) * 2 + 1;

    // Set raycaster
    this.#transformRaycaster.setFromCamera(
      {
        x: this.#mouseX,
        y: this.#mouseY,
      },
      this.#camera
    );

    // Get intersected objects
    const intersected = this.#transformRaycaster.intersectObjects(this.#objects.children);
    const uuid = intersected.length > 0 ? intersected[0].object.uuid : null;

    // Send to the main thread
    this.#postMessage({
      subject: "click_intersection",
      data: {
        uuid,
      },
    });
  }
}
