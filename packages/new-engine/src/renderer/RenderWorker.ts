import {
  Group,
  Object3D,
  ObjectLoader,
  PMREMGenerator,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
  sRGBEncoding,
} from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment";

import { Controls } from "../renderer/Controls";
import { FromRenderMessage, ToRenderMessage } from "../types";
import { disposeTree } from "../utils/disposeTree";
import { Skybox } from "./Skybox";

export type FromRenderPostMessage = (
  message: FromRenderMessage,
  transfer?: Transferable[] | undefined
) => void;

export class RenderWorker {
  scene = new Scene();
  objects: Object3D = new Group();
  renderer: WebGLRenderer | null = null;
  camera: PerspectiveCamera | null = null;

  #animationFrameId: number | null = null;
  #canvas: HTMLCanvasElement | null = null;
  #canvasWidth = 0;
  #canvasHeight = 0;

  #controls: Controls | null = null;
  #skybox = new Skybox(this.scene);

  #postMessage: FromRenderPostMessage = (...args) => {
    this.postMessage(...args);
  };

  postMessage: FromRenderPostMessage = () => {};

  constructor(canvas?: HTMLCanvasElement) {
    this.scene.add(this.objects);
    if (canvas) this.#init(canvas);
  }

  onmessage(event: MessageEvent<ToRenderMessage>) {
    const { id, subject, data } = event.data;

    this.#controls?.onmessage(event);
    this.#skybox.onmessage(event);

    switch (subject) {
      case "set_object":
        this.#setObjects(data.json);
        break;
      case "add_object":
        this.#addObject(data.json, data.parent);
        break;
      case "remove_object":
        this.#removeObject(data.uuid);
        break;
      case "move_object":
        this.#moveObject(data.uuid, data.parent);
      case "get_object":
        this.#getObject(data.uuid, id);
        break;
      case "destroy":
        this.destroy();
        break;
    }
  }

  #setObjects(json: any) {
    this.objects.removeFromParent();
    disposeTree(this.objects);

    const loader = new ObjectLoader();
    const object = loader.parse(json);

    this.scene.add(object);
    this.objects = object;
  }

  #addObject(json: any, parent?: string) {
    const loader = new ObjectLoader();
    const object = loader.parse(json);

    if (parent) {
      const parentObject = this.#getObject(parent);
      if (parentObject) {
        parentObject.add(object);
        return;
      }
    }

    this.objects.add(object);
  }

  #removeObject(uuid: string) {
    const object = this.#getObject(uuid);
    if (!object) return;

    object.removeFromParent();
    disposeTree(object);
  }

  #moveObject(uuid: string, parent: string) {
    const object = this.#getObject(uuid);
    if (!object) return;

    const parentObject = this.#getObject(parent);
    if (!parentObject) return;

    // Save object transform
    const matrix = object.matrix.clone();

    parentObject.add(object);

    // Restore object transform
    object.matrix.copy(matrix);
  }

  #getObject(uuid: string, id?: number) {
    if (uuid === "root") {
      const json = this.objects.toJSON();
      this.#postMessage({ id, subject: "got_object", data: { json } });
      return;
    }

    const object = this.objects.getObjectByProperty("uuid", uuid);

    if (id !== undefined) {
      const json = object?.toJSON();
      this.#postMessage({ id, subject: "got_object", data: { json } });
    } else {
      return object;
    }
  }

  #init(canvas: HTMLCanvasElement) {
    this.#canvas = canvas;

    // Renderer
    this.renderer = new WebGLRenderer({
      canvas,
      antialias: true,
      preserveDrawingBuffer: true,
    });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setSize(canvas.width, canvas.height);
    this.renderer.outputEncoding = sRGBEncoding;
    this.renderer.physicallyCorrectLights = true;

    // Environment
    const premGenerator = new PMREMGenerator(this.renderer);
    premGenerator.compileEquirectangularShader();
    const environment = premGenerator.fromScene(new RoomEnvironment()).texture;
    this.scene.environment = environment;

    // Camera
    this.camera = new PerspectiveCamera(75, canvas.width / canvas.height);
    this.camera.position.set(0, 0, 5);
    this.camera.lookAt(0, 0, 0);

    // Controls
    this.#controls = new Controls(this);

    // Start rendering
    this.#render();
  }

  #render() {
    this.#animationFrameId = requestAnimationFrame(() => this.#render());
    if (!this.renderer || !this.camera) return;

    this.#updateCanvasSize();
    this.renderer.render(this.scene, this.camera);
  }

  #stop() {
    if (this.#animationFrameId !== null) cancelAnimationFrame(this.#animationFrameId);
  }

  #updateCanvasSize() {
    if (!this.#canvas || !this.renderer || !this.camera) return;

    const width = this.#canvas.width;
    const height = this.#canvas.height;

    if (width === this.#canvasWidth && height === this.#canvasHeight) return;

    this.#canvasWidth = width;
    this.#canvasHeight = height;

    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
  }

  destroy() {
    // Stop rendering
    this.#stop();
    // Dispose rendering context
    this.renderer?.dispose();
    // Dispose all objects
    disposeTree(this.scene);
  }
}
